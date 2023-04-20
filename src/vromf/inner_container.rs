use std::ffi::OsString;
use std::mem::size_of;

use crate::util::debug_hex;
use crate::vromf::error::VromfError;
use crate::vromf::error::VromfError::{IndexingFileOutOfBounds, UsizeFromU64};
use crate::vromf::util::{bytes_to_int, bytes_to_long, bytes_to_usize};

pub fn decode_inner_vromf(file: &[u8]) -> Result<Vec<(String, Vec<u8>)>, VromfError> {
    // Returns slice offset from file, incrementing the ptr by offset
    let idx_file_offset = |ptr: &mut usize, offset: usize| {
        if let Some(res) = file.get(*ptr..(*ptr + offset)) {
            *ptr += offset;
            Ok(res)
        } else {
            Err(IndexingFileOutOfBounds {
                current_ptr: *ptr,
                file_size: file.len(),
                requested_len: offset,
            })
        }
    };
    let mut ptr = 0;

    let names_header = idx_file_offset(&mut ptr, size_of::<u32>())?;
    let has_digest = match names_header[0] {
        0x20 => false,
        0x30 => true,
        _ => { panic!("uh oh, unknown container type") }
    };

    let names_offset = bytes_to_int(names_header).unwrap() as usize;
    let names_count = bytes_to_int(idx_file_offset(&mut ptr, size_of::<u32>())?)? as usize;
    ptr += size_of::<u32>() * 2; // Padding to 16 byte alignment

    let data_info_offset = bytes_to_int(idx_file_offset(&mut ptr, size_of::<u32>())?)? as usize;
    let data_info_count = bytes_to_int(idx_file_offset(&mut ptr, size_of::<u32>())?)? as usize;
    ptr += size_of::<u32>() * 2; // Padding to 16 byte alignment

    if has_digest {
        let digest_end = bytes_to_long(idx_file_offset(&mut ptr, size_of::<u64>())?)?;
        let digest_begin = bytes_to_long(idx_file_offset(&mut ptr, size_of::<u64>())?)?;
        let digest_data = &file[digest_begin.try_into().unwrap()..digest_end.try_into().unwrap()];
    }

    // Names info is a set of u64s, pointing at each name
    let names_info_len = names_count * size_of::<u64>();
    let names_info = &file[names_offset..(names_offset + names_info_len)];
    let names_info_chunks = names_info.array_chunks::<{ size_of::<u64>() }>(); // No remainder from chunks as it is infallible
    let parsed_names_offsets: Vec<usize> = names_info_chunks.into_iter().map(|x| bytes_to_usize(x)).collect::<Result<_, VromfError>>()?;
    let file_names = parsed_names_offsets.into_iter().map(|start| {
        let mut buff = vec![];
        for byte in &file[start..] {
            if *byte == 0 {
                break;
            } else {
                buff.push(*byte)
            }
        }
        // The nm file has a special case, where it has additional "garbage" bytes leading in-front of it
        const NM_BYTE_ID: &[u8] = b"\xff\x3fnm";
        if let Some(leading_bytes) = buff.get(..4) {
            if leading_bytes == NM_BYTE_ID {
                buff = b"nm".to_vec();
            }
        }
        String::from_utf8(buff).unwrap()
    }).collect::<Vec<_>>();


    // FYI:
    // Each data-info-block consists of 4x u32
    // Only the first two values are used, as offset and length, the remaining two values are 0
    let data_info_len = data_info_count * size_of::<u32>() * 4; // Total length of the data-info block
    let data_info = &file[data_info_offset..(data_info_offset + data_info_len)];
    let data_info_split = data_info.array_chunks::<{ size_of::<u32>() }>(); // Data-info consists of u32 pairs, so we will split them once
    if data_info_split.remainder().len() != 0 {
        panic!("Ruh oh")
    }
    let data_info_full = data_info_split.array_chunks::<4>(); // Join together pairs of offset and length
    let data = data_info_full.map(|x|
        (u32::from_le_bytes(*x[0]) as usize, u32::from_le_bytes(*x[1]) as usize
        )).map(|(offset, size)| {
        file[offset..(offset + size)].to_vec()
    }).collect::<Vec<_>>();

    return Ok(file_names.into_iter().zip(data.into_iter()).collect::<Vec<_>>());
}


#[cfg(test)]
mod test {
    use std::fs;
    use std::time::{Duration, Instant};

    use crate::util::time;
    use crate::vromf::binary_container::decode_bin_vromf;
    use crate::vromf::inner_container::decode_inner_vromf;

    #[test]
    fn test_uncompressed() {
        let f = fs::read("./samples/checked_simple_uncompressed_checked.vromfs.bin").unwrap();
        let decoded = decode_bin_vromf(&f).unwrap();
        let inner = decode_inner_vromf(&decoded).unwrap();
    }

    #[test]
    fn test_compressed() {
        let f = fs::read("./samples/unchecked_extended_compressed_checked.vromfs.bin").unwrap();
        let decoded = decode_bin_vromf(&f).unwrap();
        let inner = decode_inner_vromf(&decoded).unwrap();
    }

    #[test]
    fn test_checked() {
        let f = fs::read("./samples/checked.vromfs").unwrap();
        let inner = decode_inner_vromf(&f).unwrap();
    }

    #[test]
    fn test_aces() {
        let f = fs::read("./samples/aces.vromfs.bin").unwrap();
        let decoded = decode_bin_vromf(&f).unwrap();
        let inner = decode_inner_vromf(&decoded).unwrap();
    }
}