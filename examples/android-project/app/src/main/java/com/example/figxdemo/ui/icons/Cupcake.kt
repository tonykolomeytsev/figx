package com.example.figxdemo.ui.icons

import androidx.compose.material3.Icon
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.graphics.StrokeJoin
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.graphics.vector.path
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import com.example.figxdemo.ui.theme.FigxDemoIcons

val FigxDemoIcons.Cupcake: ImageVector
    get() {
        if (_cupcake != null) {
            return _cupcake!!
        }
        _cupcake = ImageVector.Builder(
            name = "Cupcake",
            defaultWidth = 24.dp,
            defaultHeight = 24.dp,
            viewportWidth = 24f,
            viewportHeight = 24f,
        ).apply {
            path(
                stroke = SolidColor(Color(0xFFFFFFFF)),
                strokeLineWidth = 2f,
                strokeLineCap = StrokeCap.Round,
                strokeLineJoin = StrokeJoin.Round,
            ) {
                moveTo(19f, 14f)
                lineTo(18.1963f, 19.6263f)
                curveTo(18.1267f, 20.1134f, 18.0917f, 20.3573f, 17.9741f, 20.5405f)
                curveTo(17.8705f, 20.7019f, 17.7228f, 20.8299f, 17.5483f, 20.9097f)
                curveTo(17.3506f, 21f, 17.1051f, 21f, 16.6147f, 21f)
                lineTo(14f, 21f)
                moveTo(19f, 14f)
                lineTo(14f, 14f)
                moveTo(19f, 14f)
                curveTo(20.3031f, 13.3956f, 21f, 11.7637f, 21f, 10.3335f)
                curveTo(21f, 8.79807f, 19.9706f, 7.48314f, 18.5098f, 6.93701f)
                curveTo(18.197f, 6.8201f, 18f, 6.51809f, 18f, 6.2085f)
                curveTo(18f, 4.94284f, 16.8807f, 3.9165f, 15.5f, 3.9165f)
                curveTo(15.2737f, 3.9165f, 15.0546f, 3.94406f, 14.8462f, 3.99571f)
                curveTo(14.414f, 4.1028f, 13.9305f, 4.0791f, 13.5586f, 3.85059f)
                curveTo(12.6841f, 3.31321f, 11.6319f, 3f, 10.5f, 3f)
                curveTo(7.46243f, 3f, 5f, 5.25723f, 5f, 8.04167f)
                curveTo(5f, 8.39362f, 4.77089f, 8.71567f, 4.44287f, 8.8986f)
                curveTo(3.57772f, 9.38108f, 3f, 10.254f, 3f, 11.2502f)
                curveTo(3f, 12.5275f, 3.71228f, 13.6908f, 5f, 14f)
                moveTo(5f, 14f)
                lineTo(5.80375f, 19.6263f)
                lineTo(5.80387f, 19.6276f)
                curveTo(5.87335f, 20.1139f, 5.90813f, 20.3574f, 6.02562f, 20.5405f)
                curveTo(6.12922f, 20.7019f, 6.27719f, 20.8299f, 6.45166f, 20.9097f)
                curveTo(6.64932f, 21f, 6.89485f, 21f, 7.38528f, 21f)
                lineTo(10f, 21f)
                moveTo(5f, 14f)
                lineTo(10f, 14f)
                moveTo(10f, 14f)
                lineTo(14f, 14f)
                moveTo(10f, 14f)
                lineTo(10f, 21f)
                moveTo(14f, 14f)
                lineTo(14f, 21f)
                moveTo(14f, 21f)
                lineTo(10f, 21f)
            }
        }.build()
        return _cupcake!!
    }

private var _cupcake: ImageVector? = null

@Preview(showBackground = true)
@Composable
private fun CupcakePreview() {
    Icon(
        imageVector = FigxDemoIcons.Cupcake,
        contentDescription = null,
    )
}

