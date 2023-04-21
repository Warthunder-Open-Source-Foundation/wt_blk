#[cfg(test)]
mod test {
    use std::fs;
    use std::mem::size_of;

    #[test]
    fn extract_wrpl_1_floats() {
        let file = fs::read("./samples/decompressed wrplu_1").unwrap();

        let mut floats = vec![];
        let float_mask = &[0x11, 0x00, 0x01, 0x60];
        for (i, window) in file.windows(4).enumerate() {
            if float_mask == window {
                let all_floats = &file[(i + 4)..((i + 4) + 3 * size_of::<f32>())];

                let parsed_floats = all_floats
                    .chunks(size_of::<f32>())
                    .map(|bin| f32::from_le_bytes(bin[..4].try_into().unwrap()))
                    .map(|x| format!("{x:<15}"))
                    .collect::<Vec<_>>();
                floats.push(parsed_floats.join("\t"));
            }
        }
        fs::write("./samples/floats.txt", floats.join("\n")).unwrap();
    }
}
