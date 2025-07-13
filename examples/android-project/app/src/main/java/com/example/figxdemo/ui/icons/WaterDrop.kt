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

val FigxDemoIcons.WaterDrop: ImageVector
    get() {
        if (_waterDrop != null) {
            return _waterDrop!!
        }
        _waterDrop = ImageVector.Builder(
            name = "WaterDrop",
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
                moveTo(16.0001f, 13.3848f)
                curveTo(16.0001f, 14.6088f, 15.526f, 15.7828f, 14.6821f, 16.6483f)
                curveTo(14.203f, 17.1397f, 13.6269f, 17.5091f, 13f, 17.7364f)
                moveTo(19f, 13.6923f)
                curveTo(19f, 7.11538f, 12f, 2f, 12f, 2f)
                curveTo(12f, 2f, 5f, 7.11538f, 5f, 13.6923f)
                curveTo(5f, 15.6304f, 5.7375f, 17.4893f, 7.05025f, 18.8598f)
                curveTo(8.36301f, 20.2302f, 10.1436f, 20.9994f, 12.0001f, 20.9994f)
                curveTo(13.8566f, 20.9994f, 15.637f, 20.2298f, 16.9497f, 18.8594f)
                curveTo(18.2625f, 17.4889f, 19f, 15.6304f, 19f, 13.6923f)
                close()
            }
        }.build()
        return _waterDrop!!
    }

private var _waterDrop: ImageVector? = null

@Preview(showBackground = true)
@Composable
private fun WaterDropPreview() {
    Icon(
        imageVector = FigxDemoIcons.WaterDrop,
        contentDescription = null,
    )
}