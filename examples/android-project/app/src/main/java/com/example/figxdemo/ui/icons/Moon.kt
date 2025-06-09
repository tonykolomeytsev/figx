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

val FigxDemoIcons.Moon: ImageVector
    get() {
        if (_moon != null) {
            return _moon!!
        }
        _moon = ImageVector.Builder(
            name = "Moon",
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
                moveTo(9f, 6f)
                curveTo(9f, 10.9706f, 13.0294f, 15f, 18f, 15f)
                curveTo(18.9093f, 15f, 19.787f, 14.8655f, 20.6144f, 14.6147f)
                curveTo(19.4943f, 18.3103f, 16.0613f, 20.9999f, 12f, 20.9999f)
                curveTo(7.02944f, 20.9999f, 3f, 16.9707f, 3f, 12.0001f)
                curveTo(3f, 7.93883f, 5.69007f, 4.50583f, 9.38561f, 3.38574f)
                curveTo(9.13484f, 4.21311f, 9f, 5.09074f, 9f, 6f)
                close()
            }
        }.build()
        return _moon!!
    }

private var _moon: ImageVector? = null

@Preview(showBackground = true)
@Composable
private fun MoonPreview() {
    Icon(
        imageVector = FigxDemoIcons.Moon,
        contentDescription = null,
    )
}

