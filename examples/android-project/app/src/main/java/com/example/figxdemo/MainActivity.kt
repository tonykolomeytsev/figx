package com.example.figxdemo

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.compose.foundation.Image
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.material3.Icon
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.example.figxdemo.ui.icons.Bulb
import com.example.figxdemo.ui.icons.Cookie
import com.example.figxdemo.ui.icons.Cupcake
import com.example.figxdemo.ui.icons.FirstAid
import com.example.figxdemo.ui.icons.Leaf
import com.example.figxdemo.ui.icons.Moon
import com.example.figxdemo.ui.icons.Planet
import com.example.figxdemo.ui.icons.Puzzle
import com.example.figxdemo.ui.icons.Rainbow
import com.example.figxdemo.ui.icons.Sun
import com.example.figxdemo.ui.icons.WaterDrop
import com.example.figxdemo.ui.theme.FigxDemoIcons
import com.example.figxdemo.ui.theme.FigxDemoTheme

class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        enableEdgeToEdge()
        setContent {
            FigxDemoTheme {
                Scaffold(modifier = Modifier.fillMaxSize()) { innerPadding ->
                    Column(modifier = Modifier.padding(innerPadding)) {
                        ImportedIllustrations()
                        ImportedIcons()
                    }
                }
            }
        }
    }
}

@Composable
fun ImportedIcons(modifier: Modifier = Modifier) {
    Column(
        modifier = modifier.padding(16.dp),
        verticalArrangement = Arrangement.spacedBy(12.dp),
    ) {
        Text(text = "Icons", fontSize = 16.sp)

        Row(
            horizontalArrangement = Arrangement.spacedBy(4.dp),
        ) {
            Icon(FigxDemoIcons.Bulb, contentDescription = null)
            Icon(FigxDemoIcons.Cookie, contentDescription = null)
            Icon(FigxDemoIcons.Cupcake, contentDescription = null)
            Icon(FigxDemoIcons.FirstAid, contentDescription = null)
            Icon(FigxDemoIcons.Leaf, contentDescription = null)
            Icon(FigxDemoIcons.Moon, contentDescription = null)
            Icon(FigxDemoIcons.Planet, contentDescription = null)
            Icon(FigxDemoIcons.Puzzle, contentDescription = null)
            Icon(FigxDemoIcons.Rainbow, contentDescription = null)
            Icon(FigxDemoIcons.Sun, contentDescription = null)
            Icon(FigxDemoIcons.WaterDrop, contentDescription = null)
        }
    }
}

@Composable
fun ImportedIllustrations(modifier: Modifier = Modifier) {
    Column(
        modifier = modifier.padding(16.dp),
        verticalArrangement = Arrangement.spacedBy(12.dp),
    ) {
        Text(text = "Illustrations", fontSize = 16.sp)

        Column(
            verticalArrangement = Arrangement.spacedBy(4.dp),
        ) {
            Row {
                Image(
                    painter = painterResource(R.drawable.ill_travel),
                    contentDescription = null,
                    modifier = Modifier.size(150.dp),
                )
                Image(
                    painter = painterResource(R.drawable.ill_music),
                    contentDescription = null,
                    modifier = Modifier.size(150.dp),
                )
            }
            Row {
                Image(
                    painter = painterResource(R.drawable.ill_family),
                    contentDescription = null,
                    modifier = Modifier.size(150.dp),
                )
                Image(
                    painter = painterResource(R.drawable.ill_ecommerce),
                    contentDescription = null,
                    modifier = Modifier.size(150.dp),
                )
            }
        }
    }
}

@Preview(showBackground = true)
@Composable
fun IconsPreview() {
    FigxDemoTheme {
        ImportedIcons()
    }
}

@Preview(showBackground = true)
@Composable
fun IllustrationsPreview() {
    FigxDemoTheme {
        ImportedIllustrations()
    }
}