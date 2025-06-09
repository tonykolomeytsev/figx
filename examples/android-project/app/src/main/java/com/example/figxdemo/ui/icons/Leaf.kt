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

val FigxDemoIcons.Leaf: ImageVector
    get() {
        if (_leaf != null) {
            return _leaf!!
        }
        _leaf = ImageVector.Builder(
            name = "Leaf",
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
                moveTo(6.8291f, 17.0806f)
                curveTo(13.9002f, 21.3232f, 19.557f, 15.6663f, 18.8499f, 5.0598f)
                curveTo(8.24352f, 4.35269f, 2.58692f, 10.0097f, 6.8291f, 17.0806f)
                close()
                moveTo(6.8291f, 17.0806f)
                curveTo(6.82902f, 17.0805f, 6.82918f, 17.0807f, 6.8291f, 17.0806f)
                close()
                moveTo(6.8291f, 17.0806f)
                lineTo(5f, 18.909f)
                moveTo(6.8291f, 17.0806f)
                lineTo(10.6569f, 13.2522f)
            }
        }.build()
        return _leaf!!
    }

private var _leaf: ImageVector? = null

@Preview(showBackground = true)
@Composable
private fun LeafPreview() {
    Icon(
        imageVector = FigxDemoIcons.Leaf,
        contentDescription = null,
    )
}

