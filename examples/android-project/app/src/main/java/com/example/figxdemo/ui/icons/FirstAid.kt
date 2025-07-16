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

val FigxDemoIcons.FirstAid: ImageVector
    get() {
        if (_firstAid != null) {
            return _firstAid!!
        }
        _firstAid = ImageVector.Builder(
            name = "FirstAid",
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
                moveTo(9f, 20f)
                curveTo(9f, 20.5523f, 9.44772f, 21f, 10f, 21f)
                lineTo(14f, 21f)
                curveTo(14.5523f, 21f, 15f, 20.5523f, 15f, 20f)
                lineTo(15f, 15f)
                lineTo(20f, 15f)
                curveTo(20.5523f, 15f, 21f, 14.5523f, 21f, 14f)
                lineTo(21f, 10f)
                curveTo(21f, 9.44772f, 20.5523f, 9f, 20f, 9f)
                lineTo(15f, 9f)
                lineTo(15f, 4f)
                curveTo(15f, 3.44772f, 14.5523f, 3f, 14f, 3f)
                lineTo(10f, 3f)
                curveTo(9.44772f, 3f, 9f, 3.44772f, 9f, 4f)
                lineTo(9f, 9f)
                lineTo(4f, 9f)
                curveTo(3.44772f, 9f, 3f, 9.44772f, 3f, 10f)
                lineTo(3f, 14f)
                curveTo(3f, 14.5523f, 3.44772f, 15f, 4f, 15f)
                lineTo(9f, 15f)
                lineTo(9f, 20f)
                close()
            }
        }.build()
        return _firstAid!!
    }

private var _firstAid: ImageVector? = null

@Preview(showBackground = true)
@Composable
private fun FirstAidPreview() {
    Icon(
        imageVector = FigxDemoIcons.FirstAid,
        contentDescription = null,
    )
}