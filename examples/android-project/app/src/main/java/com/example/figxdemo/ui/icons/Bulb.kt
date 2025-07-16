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

val FigxDemoIcons.Bulb: ImageVector
    get() {
        if (_bulb != null) {
            return _bulb!!
        }
        _bulb = ImageVector.Builder(
            name = "Bulb",
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
                moveTo(9f, 21f)
                lineTo(15f, 21f)
                moveTo(12f, 3f)
                curveTo(8.68629f, 3f, 6f, 5.68629f, 6f, 9f)
                curveTo(6f, 10.2145f, 6.36084f, 11.3447f, 6.98117f, 12.2893f)
                curveTo(7.93507f, 13.7418f, 8.41161f, 14.4676f, 8.47352f, 14.5761f)
                curveTo(9.02428f, 15.541f, 8.92287f, 15.2007f, 8.99219f, 16.3096f)
                curveTo(8.99998f, 16.4342f, 9f, 16.6229f, 9f, 17f)
                curveTo(9f, 17.5523f, 9.44772f, 18f, 10f, 18f)
                lineTo(14f, 18f)
                curveTo(14.5523f, 18f, 15f, 17.5523f, 15f, 17f)
                curveTo(15f, 16.6229f, 15f, 16.4342f, 15.0078f, 16.3096f)
                curveTo(15.0771f, 15.2007f, 14.9751f, 15.541f, 15.5259f, 14.5761f)
                curveTo(15.5878f, 14.4676f, 16.0651f, 13.7418f, 17.019f, 12.2893f)
                curveTo(17.6394f, 11.3447f, 18.0002f, 10.2145f, 18.0002f, 9f)
                curveTo(18.0002f, 5.68629f, 15.3137f, 3f, 12f, 3f)
                close()
            }
        }.build()
        return _bulb!!
    }

private var _bulb: ImageVector? = null

@Preview(showBackground = true)
@Composable
private fun BulbPreview() {
    Icon(
        imageVector = FigxDemoIcons.Bulb,
        contentDescription = null,
    )
}