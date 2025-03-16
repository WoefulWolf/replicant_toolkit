use byteorder::{ByteOrder, ReadBytesExt};

pub trait ReadUtilExt: std::io::Read + std::io::Seek {
    #[inline]
    fn read_string(&mut self) -> std::io::Result<String> {
        let mut buf = String::new();
        loop {
            let byte = self.read_u8()?;
            if byte == 0 {
                break;
            }
            buf.push(byte as char);
        }

        Ok(buf)
    }

    #[inline]
    fn read_offsets<T: ByteOrder>(&mut self) -> std::io::Result<(u64, u32)> {
        let mut absolute_offset = self.stream_position()?;
        let relative_offset = self.read_u32::<T>()?;
        absolute_offset += relative_offset as u64;
        Ok((absolute_offset, relative_offset))
    }
}

impl<R: std::io::Read + std::io::Seek + ?Sized> ReadUtilExt for R {}