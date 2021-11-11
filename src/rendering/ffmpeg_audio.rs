use ffmpeg_next::{ codec, format, decoder, encoder, frame, software, Packet, Rational, Error, format::context::Output, channel_layout::ChannelLayout };

pub struct AudioTranscoder {
    pub ost_index: usize,
    pub decoder: decoder::Audio,
    pub encoder: encoder::Audio,
    pub first_frame_ts: Option<i64>,
    pub resampler: software::resampling::Context
}

impl AudioTranscoder {
    pub fn new(codec_id: codec::Id, ist: &format::stream::Stream, octx: &mut Output, ost_index: usize) -> Result<Self, Error> {
        let mut decoder = ist.codec().decoder().audio()?;
        let codec = encoder::find(codec_id).expect("failed to find encoder").audio()?;
        let global = octx.format().flags().contains(format::flag::Flags::GLOBAL_HEADER);

        decoder.set_parameters(ist.parameters())?;

        let mut output = octx.add_stream(codec)?;
        let mut encoder = output.codec().encoder().audio()?;

        let channel_layout = codec.channel_layouts().map(|cls| cls.best(decoder.channel_layout().channels())).unwrap_or(ChannelLayout::STEREO);

        if global {
            encoder.set_flags(codec::flag::Flags::GLOBAL_HEADER);
        }

        encoder.set_rate(decoder.rate() as i32);
        encoder.set_channel_layout(channel_layout);
        encoder.set_channels(channel_layout.channels());
        encoder.set_format(codec.formats().expect("unknown supported formats").next().unwrap());
        encoder.set_bit_rate(decoder.bit_rate());
        encoder.set_max_bit_rate(decoder.max_bit_rate());

        encoder.set_time_base((1, decoder.rate() as i32));
        output.set_time_base((1, decoder.rate() as i32));

        let encoder = encoder.open_as(codec)?;
        output.set_parameters(&encoder);

        let resampler = software::resampler(
            (decoder.format(), encoder.channel_layout(), decoder.rate()), // TODO source channel layout?
            (encoder.format(), encoder.channel_layout(), encoder.rate())
        )?;

        Ok(Self {
            ost_index,
            decoder,
            encoder,
            resampler,
            first_frame_ts: None
        })
    }

    pub fn receive_and_process_decoded_frames(&mut self, octx: &mut Output, ost_time_base: Rational) {
        let mut frame = frame::Audio::empty();
        let mut out_frame = frame::Audio::empty();
        
        while self.decoder.receive_frame(&mut frame).is_ok() {
            if self.first_frame_ts.is_none() {
                self.first_frame_ts = frame.timestamp();
            }

            if let Some(mut ts) = frame.timestamp() {
                ts -= self.first_frame_ts.unwrap();

                if ts >= 0 {
                    frame.set_pts(Some(ts));
                    frame.set_channel_layout(self.resampler.input().channel_layout);
        
                    let _ = self.resampler.run(&frame, &mut out_frame).unwrap();
        
                    out_frame.set_pts(Some(ts));
                    self.encoder.send_frame(&out_frame).unwrap();
        
                    self.receive_and_process_encoded_packets(octx, ost_time_base);
                }
            }
        }
    }

    pub fn receive_and_process_encoded_packets(&mut self, octx: &mut Output, ost_time_base: Rational) {
        let mut encoded = Packet::empty();
        while self.encoder.receive_packet(&mut encoded).is_ok() {
            encoded.set_stream(self.ost_index);
            encoded.rescale_ts(self.decoder.time_base(), ost_time_base);
            encoded.write_interleaved(octx).unwrap();
        }
    }
}
