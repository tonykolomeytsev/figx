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

val FigxDemoIcons.Rainbow: ImageVector
    get() {
        if (_rainbow != null) {
            return _rainbow!!
        }
        _rainbow = ImageVector.Builder(
            name = "Rainbow",
            defaultWidth = 24.dp,
            defaultHeight = 24.dp,
            viewportWidth = 24f,
            viewportHeight = 24f,
        ).apply {
            path(
                stroke = SolidColor(Color.Black),
                strokeLineWidth = 2f,
                strokeLineCap = StrokeCap.Round,
                strokeLineJoin = StrokeJoin.Round,
            ) {
                moveTo(3f, 17f)
                lineTo(3f, 15f)
                curveTo(3f, 10.0294f, 7.02944f, 6f, 12f, 6f)
                curveTo(16.9706f, 6f, 21f, 10.0294f, 21f, 15f)
                lineTo(21f, 17f)
                moveTo(6f, 17f)
                lineTo(6f, 15f)
                curveTo(6f, 11.6863f, 8.68629f, 9f, 12f, 9f)
                curveTo(15.3137f, 9f, 18f, 11.6863f, 18f, 15f)
                lineTo(18f, 17f)
                moveTo(9f, 17f)
                lineTo(9f, 15f)
                curveTo(9f, 13.3431f, 10.3431f, 12f, 12f, 12f)
                curveTo(13.6569f, 12f, 15f, 13.3431f, 15f, 15f)
                lineTo(15f, 17f)
            }
        }.build()
        return _rainbow!!
    }

private var _rainbow: ImageVector? = null

@Preview(showBackground = true)
@Composable
private fun RainbowPreview() {
    Icon(
        imageVector = FigxDemoIcons.Rainbow,
        contentDescription = null,
    )
}

