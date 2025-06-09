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

val FigxDemoIcons.Puzzle: ImageVector
    get() {
        if (_puzzle != null) {
            return _puzzle!!
        }
        _puzzle = ImageVector.Builder(
            name = "Puzzle",
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
                moveTo(20f, 7f)
                lineTo(17.8486f, 7f)
                curveTo(17.3511f, 7f, 17f, 6.49751f, 17f, 6f)
                curveTo(17f, 4.34315f, 15.6569f, 3f, 14f, 3f)
                curveTo(12.3431f, 3f, 11f, 4.34315f, 11f, 6f)
                curveTo(11f, 6.49751f, 10.6488f, 7f, 10.1513f, 7f)
                lineTo(8f, 7f)
                curveTo(7.44771f, 7f, 7f, 7.44772f, 7f, 8f)
                lineTo(7f, 10.1513f)
                curveTo(7f, 10.6488f, 6.49751f, 11f, 6f, 11f)
                curveTo(4.34315f, 11f, 3f, 12.3431f, 3f, 14f)
                curveTo(3f, 15.6569f, 4.34315f, 17f, 6f, 17f)
                curveTo(6.49751f, 17f, 7f, 17.3511f, 7f, 17.8486f)
                lineTo(7f, 20f)
                curveTo(7f, 20.5523f, 7.44771f, 21f, 8f, 21f)
                lineTo(20f, 21f)
                curveTo(20.5523f, 21f, 21f, 20.5523f, 21f, 20f)
                lineTo(21f, 17.8486f)
                curveTo(21f, 17.3511f, 20.4975f, 17f, 20f, 17f)
                curveTo(18.3431f, 17f, 17f, 15.6569f, 17f, 14f)
                curveTo(17f, 12.3431f, 18.3431f, 11f, 20f, 11f)
                curveTo(20.4975f, 11f, 21f, 10.6488f, 21f, 10.1513f)
                lineTo(21f, 8f)
                curveTo(21f, 7.44772f, 20.5523f, 7f, 20f, 7f)
                close()
            }
        }.build()
        return _puzzle!!
    }

private var _puzzle: ImageVector? = null

@Preview(showBackground = true)
@Composable
private fun PuzzlePreview() {
    Icon(
        imageVector = FigxDemoIcons.Puzzle,
        contentDescription = null,
    )
}

