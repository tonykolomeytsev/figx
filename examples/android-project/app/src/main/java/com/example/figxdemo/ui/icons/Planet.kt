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

val FigxDemoIcons.Planet: ImageVector
    get() {
        if (_planet != null) {
            return _planet!!
        }
        _planet = ImageVector.Builder(
            name = "Planet",
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
                moveTo(6.89761f, 18.1618f)
                curveTo(8.28247f, 19.3099f, 10.0607f, 20f, 12.0001f, 20f)
                curveTo(16.4184f, 20f, 20.0001f, 16.4183f, 20.0001f, 12f)
                curveTo(20.0001f, 11.431f, 19.9407f, 10.8758f, 19.8278f, 10.3404f)
                moveTo(6.89761f, 18.1618f)
                curveTo(5.12756f, 16.6944f, 4.00014f, 14.4789f, 4.00014f, 12f)
                curveTo(4.00014f, 7.58172f, 7.58186f, 4f, 12.0001f, 4f)
                curveTo(15.8494f, 4f, 19.0637f, 6.71853f, 19.8278f, 10.3404f)
                moveTo(6.89761f, 18.1618f)
                curveTo(8.85314f, 17.7147f, 11.1796f, 16.7828f, 13.526f, 15.4281f)
                curveTo(16.2564f, 13.8517f, 18.4773f, 12.0125f, 19.8278f, 10.3404f)
                moveTo(6.89761f, 18.1618f)
                curveTo(4.46844f, 18.7171f, 2.61159f, 18.5243f, 1.99965f, 17.4644f)
                curveTo(1.36934f, 16.3726f, 2.19631f, 14.5969f, 3.99999f, 12.709f)
                moveTo(19.8278f, 10.3404f)
                curveTo(21.0796f, 8.79041f, 21.5836f, 7.38405f, 21.0522f, 6.46374f)
                curveTo(20.5134f, 5.53051f, 19.0095f, 5.26939f, 16.9997f, 5.59929f)
            }
        }.build()
        return _planet!!
    }

private var _planet: ImageVector? = null

@Preview(showBackground = true)
@Composable
private fun PlanetPreview() {
    Icon(
        imageVector = FigxDemoIcons.Planet,
        contentDescription = null,
    )
}