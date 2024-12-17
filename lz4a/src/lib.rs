#![no_std]
extern crate libc;
#[cfg(test)]
#[macro_use]
extern crate std;

use libc::{c_char, c_int, c_uint, size_t};

pub type LZ4FErrorCode = size_t;
pub const LZ4F_VERSION: c_uint = 100;

extern "C" {
    // int LZ4_compress_default(const char* source, char* dest, int sourceSize, int maxDestSize);
    #[allow(non_snake_case)]
    pub fn LZ4_compress_default(
        source: *const c_char,
        dest: *mut c_char,
        sourceSize: c_int,
        maxDestSize: c_int,
    ) -> c_int;

    // int LZ4_compress_fast (const char* source, char* dest, int sourceSize, int maxDestSize, int acceleration);
    #[allow(non_snake_case)]
    pub fn LZ4_compress_fast(
        source: *const c_char,
        dest: *mut c_char,
        sourceSize: c_int,
        maxDestSize: c_int,
        acceleration: c_int,
    ) -> c_int;

    // int LZ4_compress_HC (const char* src, char* dst, int srcSize, int dstCapacity, int compressionLevel);
    #[allow(non_snake_case)]
    pub fn LZ4_compress_HC(
        src: *const c_char,
        dst: *mut c_char,
        srcSize: c_int,
        dstCapacity: c_int,
        compressionLevel: c_int,
    ) -> c_int;

    // int LZ4_decompress_safe (const char* source, char* dest, int compressedSize, int maxDecompressedSize);
    #[allow(non_snake_case)]
    pub fn LZ4_decompress_safe(
        source: *const c_char,
        dest: *mut c_char,
        compressedSize: c_int,
        maxDecompressedSize: c_int,
    ) -> c_int;

    #[allow(non_snake_case)]
    pub fn LZ4_decompress_fast(
        source: *const c_char,
        dest: *mut c_char,
        originaldSize: c_int,
    ) -> c_int;

    // const char* LZ4F_getErrorName(LZ4F_errorCode_t code);
    pub fn LZ4F_getErrorName(code: size_t) -> *const c_char;

    // int LZ4_versionNumber(void)
    pub fn LZ4_versionNumber() -> c_int;

    // int LZ4_compressBound(int isize)
    fn LZ4_compressBound(size: c_int) -> c_int;

}

const LZ4_DISTANCE_MAX: usize = 65535;

#[allow(non_snake_case)]
#[inline]
pub const fn LZ4_CompressInplaceBufferSize(decompressed: usize) -> usize {
    decompressed + LZ4_DISTANCE_MAX + 32
}

#[allow(non_snake_case)]
#[inline]
pub const fn LZ4_DecompressInplaceBufferSize(compressed: usize) -> usize {
    compressed + (compressed >> 8) + 32
}

#[allow(non_snake_case)]
#[inline]
pub fn LZ4_Decompress(src: &[u8], dst: &mut [u8]) -> i32 {
    unsafe {
        LZ4_decompress_safe(
            src.as_ptr() as *const c_char,
            dst.as_mut_ptr() as *mut c_char,
            src.len() as c_int,
            dst.len() as c_int,
        )
    }
}

#[allow(non_snake_case)]
#[inline]
pub fn LZ4_Compress(src: &[u8], dst: &mut [u8]) -> i32 {
    unsafe {
        LZ4_compress_default(
            src.as_ptr() as *const c_char,
            dst.as_mut_ptr() as *mut c_char,
            src.len() as c_int,
            dst.len() as c_int,
        )
    }
}

#[allow(non_snake_case)]
#[inline]
pub fn LZ4_CompressBounds(src: usize) -> usize {
    unsafe { LZ4_compressBound(src as c_int) as usize }
}

#[cfg(test)]
mod test {
    extern crate rand;
    use self::rand::RngCore;
    use crate::*;
    use libc::c_int;

    #[test]
    fn test_version_number() {
        let version = unsafe { LZ4_versionNumber() };
        assert_eq!(version, 10902 as c_int);

        // 640 kb original size
        assert_eq!(unsafe { LZ4_compressBound(640 * 1024) }, 657946);

        // 1Mb destination bufer
        assert_eq!(LZ4_CompressInplaceBufferSize(983009), 1024 * 1024);
        assert_eq!(LZ4_DecompressInplaceBufferSize(1044464), 1024 * 1024 - 1);
    }

    #[test]
    fn test_compression() {
        use std::vec::Vec;
        let mut rng = rand::thread_rng();

        for sz in [600_usize, 1024, 6000, 65000, 650000].iter() {
            let cz: usize = LZ4_CompressInplaceBufferSize(*sz);

            let mut orig: Vec<u8> = Vec::with_capacity(cz);
            unsafe {
                orig.set_len(cz);
                rng.fill_bytes(&mut orig[..]);

                let margin = cz - *sz;
                //compress inplace
                //maximum compressed size
                let bz = LZ4_compressBound(*sz as c_int);
                //destination compression bufer
                let mut comp: Vec<u8> = Vec::with_capacity(bz as usize);

                comp.set_len(bz as usize);

                //normal compression
                let code = LZ4_compress_default(
                    orig.as_ptr().add(margin) as *const c_char,
                    comp.as_mut_ptr() as *mut c_char,
                    (orig.len() - margin) as i32,
                    comp.len() as i32,
                );

                assert!(code >= 0);
                assert_eq!(orig.len() - margin, *sz);
                let compressed_sz = code as usize;

                //compression inplace
                let code = LZ4_compress_default(
                    orig.as_ptr().add(margin) as *const c_char,
                    orig.as_mut_ptr() as *mut c_char,
                    (orig.len() - margin) as i32,
                    orig.len() as i32,
                );

                assert!(code >= 0);

                assert_eq!(&comp[0..compressed_sz], &orig[0..compressed_sz]);
            }
        }

        assert_eq!(1, 1);
    }

    #[test]
    fn test_decompression() {
        use std::vec::Vec;
        //let mut rng = rand::thread_rng();

        for sz in [600_usize, 1024, 6000, 65000, 650000].iter() {
            let mut orig: Vec<u8> = Vec::with_capacity(*sz);
            unsafe {
                orig.set_len(*sz);
                orig.fill(0);
                {
                    //it's sort of randomized data
                    orig[0] = 1;
                    orig[*sz / 4] = 4;
                    orig[*sz / 2] = 7;
                    orig[*sz * 2 / 3] = 10;
                    orig[*sz - 1] = 1;
                }

                let bz = LZ4_compressBound(*sz as c_int) as usize;

                let mut comp: Vec<u8> = Vec::with_capacity(bz);
                comp.set_len(bz);

                let code = LZ4_compress_default(
                    orig.as_ptr() as *const c_char,
                    comp.as_mut_ptr() as *mut c_char,
                    (orig.len()) as i32,
                    (bz) as i32,
                );

                assert!(code > 0);
                //size of compressed data
                println!(
                    "orig {}; compressed {}; in buf len {}",
                    *sz,
                    code as usize,
                    comp.len()
                );
                //compressed size
                let cz = code as usize;

                let mut buf: Vec<u8> = Vec::with_capacity(*sz);
                buf.set_len(*sz);

                let code = LZ4_decompress_safe(
                    comp.as_ptr() as *const c_char,
                    buf.as_mut_ptr() as *mut c_char,
                    cz as i32,
                    *sz as i32,
                );

                assert!(code > 0);

                let cz = code as usize;

                assert_eq!(cz, *sz);
                assert_eq!(&orig[0..*sz], &buf[0..cz]);
            }
        }
    }
}
