#![allow(unused)]

use crate::boxed::{WebpBox, wrap_bytes};
use crate::error::WebPSimpleError;
use libwebp_sys as sys;
use std::os::raw::*;
use std::ptr;

/// Return the encoder's version number, packed in hexadecimal using 8bits for
/// each of major/minor/revision.
///
/// E.g: v2.5.7 is 0x020507.
///
/// ## Examples
///
/// ```rust
/// use libwebp::WebPGetEncoderVersion;
///
/// let version = WebPGetEncoderVersion();
/// ```
#[allow(non_snake_case)]
pub fn WebPGetEncoderVersion() -> u32 {
    (unsafe { sys::WebPGetEncoderVersion() }) as u32
}

fn encode_size_check(len: usize, width: u32, height: u32, stride: u32, pixelwidth: usize) {
    assert_eq!(
        width as c_int as u32, width,
        "width {} not within c_int",
        width
    );
    assert_eq!(
        height as c_int as u32, height,
        "height {} not within c_int",
        height
    );
    assert_eq!(
        stride as c_int as u32, stride,
        "stride {} not within c_int",
        stride
    );
    let width = width as usize;
    let height = height as usize;
    let equal = if width == 0 {
        len == 0
    } else {
        len % pixelwidth == 0 && len / pixelwidth % width == 0 && len / pixelwidth / width == height
    };
    assert!(
        equal,
        "buffer size mismatch: {} * {} * {} != {}",
        width, height, pixelwidth, len
    );
}

/// Same as [`WebPEncodeRGBA`], but expecting R, G, B, R, G, B...
/// ordered data.
///
/// [`WebPEncodeRGBA`]: fn.WebPEncodeRGBA.html
#[allow(non_snake_case)]
pub fn WebPEncodeRGB(
    rgb: &[u8],
    width: u32,
    height: u32,
    stride: u32,
    quality_factor: f32,
) -> Result<WebpBox<[u8]>, WebPSimpleError> {
    encode_size_check(rgb.len(), width, height, stride, 3);
    let mut output: *mut u8 = ptr::null_mut();
    let result = unsafe {
        sys::WebPEncodeRGB(
            rgb.as_ptr(),
            width as c_int,
            height as c_int,
            stride as c_int,
            quality_factor as c_float,
            &mut output,
        )
    };
    if result != 0 {
        unsafe { wrap_bytes(output, || result) }
    } else {
        Err(WebPSimpleError)
    }
}

/// Same as [`WebPEncodeRGBA`], but expecting B, G, R, B, G, R...
/// ordered data.
///
/// [`WebPEncodeRGBA`]: fn.WebPEncodeRGBA.html
#[allow(non_snake_case)]
pub fn WebPEncodeBGR(
    bgr: &[u8],
    width: u32,
    height: u32,
    stride: u32,
    quality_factor: f32,
) -> Result<WebpBox<[u8]>, WebPSimpleError> {
    encode_size_check(bgr.len(), width, height, stride, 3);
    let mut output: *mut u8 = ptr::null_mut();
    let result = unsafe {
        sys::WebPEncodeBGR(
            bgr.as_ptr(),
            width as c_int,
            height as c_int,
            stride as c_int,
            quality_factor as c_float,
            &mut output,
        )
    };
    if result != 0 {
        unsafe { wrap_bytes(output, || result) }
    } else {
        Err(WebPSimpleError)
    }
}

/// Encodes images pointed to by `rgba` and returns the WebP binary data.
///
/// The ordering of samples in memory is R, G, B, A, R, G, B, A... in scan
/// order (endian-independent).
///
/// ## Compression
///
/// This function compresses using the lossy format, and the `quality_factor`
/// can go from 0 (smaller output, lower quality) to 100 (best quality,
/// larger output).
///
/// For lossless compression, you can use the [`WebPEncodeLosslessRGBA`]
/// family.
///
/// [`WebPEncodeLosslessRGBA`]: fn.WebPEncodeLosslessRGBA.html
///
/// ## Errors
///
/// Returns `Err` if `data` doesn't contain a valid WebP header.
///
/// ## Panics
///
/// Panics when `stride` is too small or `rgba` has a wrong size.
///
/// ## Variants
///
/// - `WebPEncodeRGBA`
/// - [`WebPEncodeBGRA`]
/// - [`WebPEncodeRGB`]
/// - [`WebPEncodeBGR`]
///
/// [`WebPEncodeBGRA`]: fn.WebPEncodeBGRA.html
/// [`WebPEncodeRGB`]: fn.WebPEncodeRGB.html
/// [`WebPEncodeBGR`]: fn.WebPEncodeBGR.html
///
/// ## Examples
///
/// ```rust
/// use libwebp::WebPEncodeRGBA;
///
/// let buf: &[u8] = &[
///     255, 255, 255, 255, // white
///     255, 0, 0, 255, // red
///     0, 255, 0, 255, // green
///     0, 0, 255, 255, // blue
/// ];
/// let data = WebPEncodeRGBA(buf, 2, 2, 8, 75.0).unwrap();
/// assert_eq!(&data[..4], b"RIFF");
/// assert_eq!(&data[8..12], b"WEBP");
/// ```
#[allow(non_snake_case)]
pub fn WebPEncodeRGBA(
    rgba: &[u8],
    width: u32,
    height: u32,
    stride: u32,
    quality_factor: f32,
) -> Result<WebpBox<[u8]>, WebPSimpleError> {
    encode_size_check(rgba.len(), width, height, stride, 4);
    let mut output: *mut u8 = ptr::null_mut();
    let result = unsafe {
        sys::WebPEncodeRGBA(
            rgba.as_ptr(),
            width as c_int,
            height as c_int,
            stride as c_int,
            quality_factor as c_float,
            &mut output,
        )
    };
    if result != 0 {
        unsafe { wrap_bytes(output, || result) }
    } else {
        Err(WebPSimpleError)
    }
}

/// Same as [`WebPEncodeRGBA`], but expecting B, G, R, A, B, G, R, A...
/// ordered data.
///
/// [`WebPEncodeRGBA`]: fn.WebPEncodeRGBA.html
#[allow(non_snake_case)]
pub fn WebPEncodeBGRA(
    bgra: &[u8],
    width: u32,
    height: u32,
    stride: u32,
    quality_factor: f32,
) -> Result<WebpBox<[u8]>, WebPSimpleError> {
    encode_size_check(bgra.len(), width, height, stride, 4);
    let mut output: *mut u8 = ptr::null_mut();
    let result = unsafe {
        sys::WebPEncodeBGRA(
            bgra.as_ptr(),
            width as c_int,
            height as c_int,
            stride as c_int,
            quality_factor as c_float,
            &mut output,
        )
    };
    if result != 0 {
        unsafe { wrap_bytes(output, || result) }
    } else {
        Err(WebPSimpleError)
    }
}

/// Same as [`WebPEncodeLosslessRGBA`], but expecting R, G, B, R, G, B...
/// ordered data.
///
/// [`WebPEncodeLosslessRGBA`]: fn.WebPEncodeLosslessRGBA.html
#[allow(non_snake_case)]
pub fn WebPEncodeLosslessRGB(
    rgb: &[u8],
    width: u32,
    height: u32,
    stride: u32,
) -> Result<WebpBox<[u8]>, WebPSimpleError> {
    encode_size_check(rgb.len(), width, height, stride, 3);
    let mut output: *mut u8 = ptr::null_mut();
    let result = unsafe {
        sys::WebPEncodeLosslessRGB(
            rgb.as_ptr(),
            width as c_int,
            height as c_int,
            stride as c_int,
            &mut output,
        )
    };
    if result != 0 {
        unsafe { wrap_bytes(output, || result) }
    } else {
        Err(WebPSimpleError)
    }
}

/// Same as [`WebPEncodeLosslessRGBA`], but expecting B, G, R, B, G, R...
/// ordered data.
///
/// [`WebPEncodeLosslessRGBA`]: fn.WebPEncodeLosslessRGBA.html
#[allow(non_snake_case)]
pub fn WebPEncodeLosslessBGR(
    bgr: &[u8],
    width: u32,
    height: u32,
    stride: u32,
) -> Result<WebpBox<[u8]>, WebPSimpleError> {
    encode_size_check(bgr.len(), width, height, stride, 3);
    let mut output: *mut u8 = ptr::null_mut();
    let result = unsafe {
        sys::WebPEncodeLosslessBGR(
            bgr.as_ptr(),
            width as c_int,
            height as c_int,
            stride as c_int,
            &mut output,
        )
    };
    if result != 0 {
        unsafe { wrap_bytes(output, || result) }
    } else {
        Err(WebPSimpleError)
    }
}

/// Encodes images pointed to by `rgba` and returns the WebP binary data.
///
/// The ordering of samples in memory is R, G, B, A, R, G, B, A... in scan
/// order (endian-independent).
///
/// ## Compression
///
/// This function compresses using the lossless format. Files are usually
/// larger than lossy format, but will not suffer any compression loss.
///
/// For lossy compression, you can use the [`WebPEncodeRGBA`] family.
///
/// [`WebPEncodeRGBA`]: fn.WebPEncodeRGBA.html
///
/// Note these functions, like the lossy versions, use the library's default
/// settings. For lossless this means `exact` is disabled. RGB values in
/// transparent areas will be modified to improve compression. To avoid this,
/// use `WebPEncode()` and set `WebPConfig::exact` to `1`.
///
/// (The Rust binding does not yet have the corresponding function.)
///
/// ## Errors
///
/// Returns `Err` if `data` doesn't contain a valid WebP header.
///
/// ## Panics
///
/// Panics when `stride` is too small or `rgba` has a wrong size.
///
/// ## Variants
///
/// - `WebPEncodeLosslessRGBA`
/// - [`WebPEncodeLosslessBGRA`]
/// - [`WebPEncodeLosslessRGB`]
/// - [`WebPEncodeLosslessBGR`]
///
/// [`WebPEncodeLosslessBGRA`]: fn.WebPEncodeLosslessBGRA.html
/// [`WebPEncodeLosslessRGB`]: fn.WebPEncodeLosslessRGB.html
/// [`WebPEncodeLosslessBGR`]: fn.WebPEncodeLosslessBGR.html
///
/// ## Examples
///
/// ```rust
/// use libwebp::WebPEncodeLosslessRGBA;
///
/// let buf: &[u8] = &[
///     255, 255, 255, 255, // white
///     255, 0, 0, 255, // red
///     0, 255, 0, 255, // green
///     0, 0, 255, 255, // blue
/// ];
/// let data = WebPEncodeLosslessRGBA(buf, 2, 2, 8).unwrap();
/// assert_eq!(&data[..4], b"RIFF");
/// assert_eq!(&data[8..12], b"WEBP");
/// ```
#[allow(non_snake_case)]
pub fn WebPEncodeLosslessRGBA(
    rgba: &[u8],
    width: u32,
    height: u32,
    stride: u32,
) -> Result<WebpBox<[u8]>, WebPSimpleError> {
    encode_size_check(rgba.len(), width, height, stride, 4);
    let mut output: *mut u8 = ptr::null_mut();
    let result = unsafe {
        sys::WebPEncodeLosslessRGBA(
            rgba.as_ptr(),
            width as c_int,
            height as c_int,
            stride as c_int,
            &mut output,
        )
    };
    if result != 0 {
        unsafe { wrap_bytes(output, || result) }
    } else {
        Err(WebPSimpleError)
    }
}

/// Same as [`WebPEncodeLosslessRGBA`], but expecting B, G, R, A, B, G, R, A...
/// ordered data.
///
/// [`WebPEncodeLosslessRGBA`]: fn.WebPEncodeLosslessRGBA.html
#[allow(non_snake_case)]
pub fn WebPEncodeLosslessBGRA(
    bgra: &[u8],
    width: u32,
    height: u32,
    stride: u32,
) -> Result<WebpBox<[u8]>, WebPSimpleError> {
    encode_size_check(bgra.len(), width, height, stride, 4);
    let mut output: *mut u8 = ptr::null_mut();
    let result = unsafe {
        sys::WebPEncodeLosslessBGRA(
            bgra.as_ptr(),
            width as c_int,
            height as c_int,
            stride as c_int,
            &mut output,
        )
    };
    if result != 0 {
        unsafe { wrap_bytes(output, || result) }
    } else {
        Err(WebPSimpleError)
    }
}
