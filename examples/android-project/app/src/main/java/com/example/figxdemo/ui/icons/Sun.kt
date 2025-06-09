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

val FigxDemoIcons.Sun: ImageVector
    get() {
        if (_sun != null) {
            return _sun!!
        }
        _sun = ImageVector.Builder(
            name = "Sun",
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
                moveTo(12f, 4f)
                lineTo(12f, 2f)
                moveTo(12f, 20f)
                lineTo(12f, 22f)
                moveTo(6.41421f, 6.41421f)
                lineTo(5f, 5f)
                moveTo(17.728f, 17.728f)
                lineTo(19.1422f, 19.1422f)
                moveTo(4f, 12f)
                lineTo(2f, 12f)
                moveTo(20f, 12f)
                lineTo(22f, 12f)
                moveTo(17.7285f, 6.41421f)
                lineTo(19.1427f, 5f)
                moveTo(6.4147f, 17.728f)
                lineTo(5.00049f, 19.1422f)
                moveTo(12f, 17f)
                curveTo(9.23858f, 17f, 7f, 14.7614f, 7f, 12f)
                curveTo(7f, 9.23858f, 9.23858f, 7f, 12f, 7f)
                curveTo(14.7614f, 7f, 17f, 9.23858f, 17f, 12f)
                curveTo(17f, 14.7614f, 14.7614f, 17f, 12f, 17f)
                close()
            }
        }.build()
        return _sun!!
    }

private var _sun: ImageVector? = null

@Preview(showBackground = true)
@Composable
private fun SunPreview() {
    Icon(
        imageVector = FigxDemoIcons.Sun,
        contentDescription = null,
    )
}

