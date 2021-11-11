import QtQuick 2.15
import QtQuick.Controls 2.15 as QQC

QQC.ToolTip {
    id: root;
    delay: 500;
    background: Rectangle {
        color: styleButtonColor;
        border.width: 1 * dpiScale;
        border.color: stylePopupBorder
        radius: 4 * dpiScale;
    }
    enter: Transition {
        NumberAnimation { property: "y"; from: -height/1.3 - bottomMargin; to: -height - bottomMargin; easing.type: Easing.OutExpo; duration: 500; }
        NumberAnimation { property: "opacity"; from: 0.0; to: 1.0; easing.type: Easing.OutExpo; duration: 500; }
    }
    exit: Transition {
        NumberAnimation { property: "y"; from: -height - bottomMargin; to: -height/1.3 - bottomMargin; easing.type: Easing.OutExpo; duration: 500; }
        NumberAnimation { property: "opacity"; from: 1.0; to: 0.0; easing.type: Easing.OutExpo; duration: 500; }
    }
    contentItem: Text {
        id: infotxt2;
        font.pixelSize: 12 * dpiScale;
        text: root.text;
        color: styleTextColor;
    }
    bottomMargin: 5 * dpiScale;
    topPadding: 5 * dpiScale;
    rightPadding: 8 * dpiScale;
    bottomPadding: topPadding;
    leftPadding: rightPadding;
}
