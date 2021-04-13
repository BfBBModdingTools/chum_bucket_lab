use std::{
    fs::{File, OpenOptions},
    io,
    io::{Read, Result, Seek, SeekFrom, Write},
};

use byteorder::{ReadBytesExt, WriteBytesExt, LE};

fn pad_to_exact(v: &mut Vec<u8>, to: usize) {
    while v.len() < to {
        v.push(0u8);
    }
}

fn pad_to_nearest(v: &mut Vec<u8>, to: usize) {
    while v.len() % to != 0 {
        v.push(0u8);
    }
}

#[derive(Default, Debug)]
pub struct XBE {
    pub image_header: ImageHeader,
    pub certificate: Certificate,
    pub section_headers: Vec<SectionHeader>,
    pub section_names: Vec<String>,
    pub library_version: Vec<LibraryVersion>,
    pub debug_pathname: String,
    pub debug_filename: String,
    pub debug_unicode_filename: Vec<u16>,
    pub logo_bitmap: LogoBitmap,
    pub sections: Vec<Section>,
}

impl XBE {
    /// Serialize this XBE object to a valid .xbe executable
    ///
    /// Note: this currently results in an xbe file with less ending padding
    /// when tested with SpongeBob SquarePants: Battle for Bikini Bottom,
    /// but the outputted xbe works regardless.
    pub fn serialize(&self) -> Result<Vec<u8>> {
        let mut img_hdr_v = self.image_header.serialize()?;
        let mut ctf_v = self.certificate.serialize()?;
        let mut sec_hdrs = self.serialize_section_headers()?;
        let mut sec_names = self.serialize_section_names()?;
        let mut library_versions = self.serialize_library_versions()?;
        let mut bitmap = self.logo_bitmap.serialize()?;
        let mut sections = self.serialize_sections()?;

        pad_to_exact(
            &mut &mut img_hdr_v,
            (self.image_header.certificate_address - self.image_header.base_address) as usize,
        );
        img_hdr_v.append(&mut ctf_v);

        pad_to_exact(
            &mut img_hdr_v,
            (self.image_header.section_headers_address - self.image_header.base_address) as usize,
        );
        img_hdr_v.append(&mut sec_hdrs);

        pad_to_exact(
            &mut img_hdr_v,
            (self.section_headers[0].section_name_address - self.image_header.base_address)
                as usize,
        );
        img_hdr_v.append(&mut sec_names);

        pad_to_nearest(&mut img_hdr_v, 4);
        img_hdr_v.append(&mut library_versions);

        // Write Debug file/path names
        pad_to_exact(
            &mut img_hdr_v,
            (self.image_header.debug_unicode_filename_address - self.image_header.base_address)
                as usize,
        );

        for x in self.debug_unicode_filename.iter() {
            img_hdr_v.write_u16::<LE>(*x)?;
        }

        // debug filename is part of this string, just starting at a later offset
        pad_to_exact(
            &mut img_hdr_v,
            (self.image_header.debug_pathname_address - self.image_header.base_address) as usize,
        );
        img_hdr_v.write(self.debug_pathname.as_bytes())?;

        // Write bitmap
        pad_to_exact(
            &mut img_hdr_v,
            (self.image_header.logo_bitmap_address - self.image_header.base_address) as usize,
        );
        img_hdr_v.append(&mut bitmap);

        // Pad header
        pad_to_nearest(&mut img_hdr_v, 0x1000);

        // Add sections
        img_hdr_v.append(&mut sections);

        // End padding
        pad_to_nearest(&mut img_hdr_v, 0x1000);

        Ok(img_hdr_v)
    }

    pub fn serialize_section_headers(&self) -> Result<Vec<u8>> {
        let mut v = vec![];
        for hdr in self.section_headers.iter() {
            v.append(&mut hdr.serialize()?);
        }
        Ok(v)
    }

    pub fn serialize_section_names(&self) -> Result<Vec<u8>> {
        let mut v = vec![];

        for n in self.section_names.iter() {
            v.write(&n.as_bytes())?;
        }

        Ok(v)
    }

    pub fn serialize_library_versions(&self) -> Result<Vec<u8>> {
        let mut v = vec![];

        for l in self.library_version.iter() {
            v.append(&mut l.serialize()?);
        }

        Ok(v)
    }

    pub fn serialize_sections(&self) -> Result<Vec<u8>> {
        let mut v = vec![];

        if self.section_headers.len() != self.sections.len() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Number of section headers does not match number of sections.",
            ));
        }

        // sort headers by raw address
        // TODO: This currently makes some assumptions that may or may not be true.
        // it doesn't actually ensure that the raw_address field of the section header is
        // where the section is actually placed. Instead it places the sections order from
        // lowest raw address to highest and pads them to the next 0x1000 bytes.
        // This approach works for BfBB but may not for other xbes
        let mut sorted_headers = vec![];
        for i in 0..self.section_headers.len() {
            sorted_headers.push((&self.section_headers[i], &self.sections[i]));
        }
        sorted_headers.sort_by(|a, b| {
            if a.0.raw_address > b.0.raw_address {
                std::cmp::Ordering::Greater
            } else if a.0.raw_address == b.0.raw_address {
                std::cmp::Ordering::Equal
            } else {
                std::cmp::Ordering::Less
            }
        });

        for (_, sec) in sorted_headers {
            // let s = &self.sections[i];
            v.append(&mut sec.serialize()?);
            pad_to_nearest(&mut v, 0x1000);
        }

        Ok(v)
    }
}

#[derive(Debug)]
pub struct ImageHeader {
    pub magic_number: [u8; 4],
    pub digital_signature: [u8; 256],
    pub base_address: u32,
    pub size_of_headers: u32,
    pub size_of_image: u32, // Size of virtual address space
    pub size_of_image_header: u32,
    pub time_date: u32,
    pub certificate_address: u32,
    pub number_of_sections: u32,
    pub section_headers_address: u32,
    pub initialization_flags: u32,
    pub entry_point: u32,
    pub tls_address: u32,
    pub pe_stack_commit: u32,
    pub pe_heap_reserve: u32,
    pub pe_head_commit: u32,
    pub pe_base_address: u32,
    pub pe_size_of_image: u32,
    pub pe_checksum: u32,
    pub pe_time_date: u32,
    pub debug_pathname_address: u32,
    pub debug_filename_address: u32,
    pub debug_unicode_filename_address: u32,
    pub kernel_image_thunk_address: u32,
    pub non_kernel_import_directory_address: u32,
    pub number_of_library_versions: u32,
    pub library_versions_address: u32,
    pub kernel_library_version_address: u32,
    pub xapi_library_version_address: u32,
    pub logo_bitmap_address: u32,
    pub logo_bitmap_size: u32,
}

impl ImageHeader {
    pub fn serialize(&self) -> Result<Vec<u8>> {
        let mut v = vec![];

        v.write(&self.magic_number)?;
        v.write(&self.digital_signature)?;
        v.write_u32::<LE>(self.base_address)?;
        v.write_u32::<LE>(self.size_of_headers)?;
        v.write_u32::<LE>(self.size_of_image)?;
        v.write_u32::<LE>(self.size_of_image_header)?;
        v.write_u32::<LE>(self.time_date)?;
        v.write_u32::<LE>(self.certificate_address)?;
        v.write_u32::<LE>(self.number_of_sections)?;
        v.write_u32::<LE>(self.section_headers_address)?;
        v.write_u32::<LE>(self.initialization_flags)?;
        v.write_u32::<LE>(self.entry_point)?;
        v.write_u32::<LE>(self.tls_address)?;
        v.write_u32::<LE>(self.pe_stack_commit)?;
        v.write_u32::<LE>(self.pe_heap_reserve)?;
        v.write_u32::<LE>(self.pe_head_commit)?;
        v.write_u32::<LE>(self.pe_base_address)?;
        v.write_u32::<LE>(self.pe_size_of_image)?;
        v.write_u32::<LE>(self.pe_checksum)?;
        v.write_u32::<LE>(self.pe_time_date)?;
        v.write_u32::<LE>(self.debug_pathname_address)?;
        v.write_u32::<LE>(self.debug_filename_address)?;
        v.write_u32::<LE>(self.debug_unicode_filename_address)?;
        v.write_u32::<LE>(self.kernel_image_thunk_address)?;
        v.write_u32::<LE>(self.non_kernel_import_directory_address)?;
        v.write_u32::<LE>(self.number_of_library_versions)?;
        v.write_u32::<LE>(self.library_versions_address)?;
        v.write_u32::<LE>(self.kernel_library_version_address)?;
        v.write_u32::<LE>(self.xapi_library_version_address)?;
        v.write_u32::<LE>(self.logo_bitmap_address)?;
        v.write_u32::<LE>(self.logo_bitmap_size)?;

        while v.len() < self.size_of_image_header as usize {
            v.write_u8(0)?;
        }

        Ok(v)
    }
}

impl Default for ImageHeader {
    fn default() -> Self {
        ImageHeader {
            magic_number: [0u8; 4],
            digital_signature: [0u8; 256],
            base_address: 0,
            size_of_headers: 0,
            size_of_image: 0,
            size_of_image_header: 0,
            time_date: 0,
            certificate_address: 0,
            number_of_sections: 0,
            section_headers_address: 0,
            initialization_flags: 0,
            entry_point: 0,
            tls_address: 0,
            pe_stack_commit: 0,
            pe_heap_reserve: 0,
            pe_head_commit: 0,
            pe_base_address: 0,
            pe_size_of_image: 0,
            pe_checksum: 0,
            pe_time_date: 0,
            debug_pathname_address: 0,
            debug_filename_address: 0,
            debug_unicode_filename_address: 0,
            kernel_image_thunk_address: 0,
            non_kernel_import_directory_address: 0,
            number_of_library_versions: 0,
            library_versions_address: 0,
            kernel_library_version_address: 0,
            xapi_library_version_address: 0,
            logo_bitmap_address: 0,
            logo_bitmap_size: 0,
        }
    }
}

#[derive(Debug)]
pub struct Certificate {
    pub size: u32,
    pub time_date: u32,
    pub title_id: u32,
    pub title_name: [u8; 0x50],
    pub alternate_title_ids: [u8; 0x40],
    pub allowed_media: u32,
    pub game_region: u32,
    pub game_ratings: u32,
    pub disk_number: u32,
    pub version: u32,
    pub lan_key: [u8; 0x10],
    pub signature_key: [u8; 0x10],
    pub alternate_signature_keys: [u8; 0x100],
    pub reserved: Vec<u8>, //There seems to be more bytes I can't find any documentation on.
}

impl Certificate {
    pub fn serialize(&self) -> Result<Vec<u8>> {
        let mut v = vec![];

        v.write_u32::<LE>(self.size)?;
        v.write_u32::<LE>(self.time_date)?;
        v.write_u32::<LE>(self.title_id)?;
        v.write(&self.title_name)?;
        v.write(&self.alternate_title_ids)?;
        v.write_u32::<LE>(self.allowed_media)?;
        v.write_u32::<LE>(self.game_region)?;
        v.write_u32::<LE>(self.game_ratings)?;
        v.write_u32::<LE>(self.disk_number)?;
        v.write_u32::<LE>(self.version)?;
        v.write(&self.lan_key)?;
        v.write(&self.signature_key)?;
        v.write(&self.alternate_signature_keys)?;
        v.write(&self.reserved)?;

        Ok(v)
    }
}

impl Default for Certificate {
    fn default() -> Self {
        Certificate {
            size: 0,
            time_date: 0,
            title_id: 0,
            title_name: [0u8; 0x50],
            alternate_title_ids: [0u8; 0x40],
            allowed_media: 0,
            game_region: 0,
            game_ratings: 0,
            disk_number: 0,
            version: 0,
            lan_key: [0u8; 16],
            signature_key: [0u8; 16],
            alternate_signature_keys: [0u8; 0x100],
            reserved: vec![],
        }
    }
}

#[derive(Debug, Default)]
pub struct LogoBitmap {
    pub bitmap: Vec<u8>,
}

impl LogoBitmap {
    pub fn serialize(&self) -> Result<Vec<u8>> {
        Ok(self.bitmap.clone())
    }
}

#[derive(Debug, Default)]
pub struct SectionHeader {
    pub section_flags: u32,
    pub virtual_address: u32,
    pub virtual_size: u32,
    pub raw_address: u32,
    pub raw_size: u32,
    pub section_name_address: u32,
    pub section_name_reference_count: u32,
    pub head_shared_page_reference_count_address: u32,
    pub tail_shared_page_reference_count_address: u32,
    pub section_digest: [u8; 0x14],
}

impl SectionHeader {
    pub fn serialize(&self) -> Result<Vec<u8>> {
        let mut v = vec![];

        v.write_u32::<LE>(self.section_flags)?;
        v.write_u32::<LE>(self.virtual_address)?;
        v.write_u32::<LE>(self.virtual_size)?;
        v.write_u32::<LE>(self.raw_address)?;
        v.write_u32::<LE>(self.raw_size)?;
        v.write_u32::<LE>(self.section_name_address)?;
        v.write_u32::<LE>(self.section_name_reference_count)?;
        v.write_u32::<LE>(self.head_shared_page_reference_count_address)?;
        v.write_u32::<LE>(self.tail_shared_page_reference_count_address)?;
        v.write(&self.section_digest)?;

        Ok(v)
    }
}

#[derive(Debug, Default)]
pub struct LibraryVersion {
    pub library_name: [u8; 8],
    pub major_version: u16,
    pub minor_version: u16,
    pub build_version: u16,
    pub library_flags: u16,
}

impl LibraryVersion {
    pub fn serialize(&self) -> Result<Vec<u8>> {
        let mut v = vec![];

        v.write(&self.library_name)?;
        v.write_u16::<LE>(self.major_version)?;
        v.write_u16::<LE>(self.minor_version)?;
        v.write_u16::<LE>(self.build_version)?;
        v.write_u16::<LE>(self.library_flags)?;

        Ok(v)
    }
}

#[derive(Debug, Default)]
pub struct TLS {
    pub data_start_address: u32,
    pub data_end_address: u32,
    pub tls_index_address: u32,
    pub tls_callback_address: u32,
    pub size_of_zero_fill: u32,
    pub characteristics: u32,
}

#[derive(Debug, Default)]
pub struct Section {
    pub bytes: Vec<u8>,
}

impl Section {
    pub fn serialize(&self) -> Result<Vec<u8>> {
        Ok(self.bytes.clone())
    }
}

pub fn load_xbe(mut file: File) -> std::io::Result<XBE> {
    // let mut xbe = XBE::default();

    // Read header data
    let image_header = load_image_header(&mut file)?;

    // Read certificate data
    let certificate = load_certificate(&mut file, &image_header)?;

    // Read logo bitmap data
    let logo_bitmap = load_logo_bitmap(&mut file, &image_header)?;

    // Read section data
    let section_headers = load_section_headers(&mut file, &image_header)?;
    let section_names = load_section_names(&mut file, &image_header, &section_headers)?;

    // Read debug path data
    let debug_filename = load_debug_filename(&mut file, &image_header)?;
    let debug_pathname = load_debug_pathname(&mut file, &image_header)?;
    let debug_unicode_filename = load_debug_unicode_filename(&mut file, &image_header)?;

    // Read sections
    let mut sections = vec![];
    for sec_hdr in section_headers.iter() {
        sections.push(load_section(&mut file, sec_hdr)?);
    }

    // Read library versions
    let library_version = load_library_versions(&mut file, &image_header)?;
    Ok(XBE {
        image_header,
        certificate,
        section_headers,
        section_names,
        library_version,
        debug_filename,
        debug_pathname,
        debug_unicode_filename,
        logo_bitmap,
        sections,
    })
}

fn load_image_header(file: &mut File) -> Result<ImageHeader> {
    let mut header = ImageHeader::default();

    file.read_exact(&mut header.magic_number)?;
    file.read_exact(&mut header.digital_signature)?;
    header.base_address = file.read_u32::<LE>()?;
    header.size_of_headers = file.read_u32::<LE>()?;
    header.size_of_image = file.read_u32::<LE>()?;
    header.size_of_image_header = file.read_u32::<LE>()?;
    header.time_date = file.read_u32::<LE>()?;
    header.certificate_address = file.read_u32::<LE>()?;
    header.number_of_sections = file.read_u32::<LE>()?;
    header.section_headers_address = file.read_u32::<LE>()?;
    header.initialization_flags = file.read_u32::<LE>()?;
    header.entry_point = file.read_u32::<LE>()?;
    header.tls_address = file.read_u32::<LE>()?;
    header.pe_stack_commit = file.read_u32::<LE>()?;
    header.pe_heap_reserve = file.read_u32::<LE>()?;
    header.pe_head_commit = file.read_u32::<LE>()?;
    header.pe_base_address = file.read_u32::<LE>()?;
    header.pe_size_of_image = file.read_u32::<LE>()?;
    header.pe_checksum = file.read_u32::<LE>()?;
    header.pe_time_date = file.read_u32::<LE>()?;
    header.debug_pathname_address = file.read_u32::<LE>()?;
    header.debug_filename_address = file.read_u32::<LE>()?;
    header.debug_unicode_filename_address = file.read_u32::<LE>()?;
    header.kernel_image_thunk_address = file.read_u32::<LE>()?;
    header.non_kernel_import_directory_address = file.read_u32::<LE>()?;
    header.number_of_library_versions = file.read_u32::<LE>()?;
    header.library_versions_address = file.read_u32::<LE>()?;
    header.kernel_library_version_address = file.read_u32::<LE>()?;
    header.xapi_library_version_address = file.read_u32::<LE>()?;
    header.logo_bitmap_address = file.read_u32::<LE>()?;
    header.logo_bitmap_size = file.read_u32::<LE>()?;
    Ok(header)
}

fn load_certificate(file: &mut File, header: &ImageHeader) -> Result<Certificate> {
    let start = (header.certificate_address - header.base_address) as u64;
    file.seek(SeekFrom::Start(start))?;

    let mut certificate = Certificate::default();

    certificate.size = file.read_u32::<LE>()?;
    certificate.time_date = file.read_u32::<LE>()?;
    certificate.title_id = file.read_u32::<LE>()?;
    file.read_exact(&mut certificate.title_name)?;
    file.read_exact(&mut certificate.alternate_title_ids)?;
    certificate.allowed_media = file.read_u32::<LE>()?;
    certificate.game_region = file.read_u32::<LE>()?;
    certificate.game_ratings = file.read_u32::<LE>()?;
    certificate.disk_number = file.read_u32::<LE>()?;
    certificate.version = file.read_u32::<LE>()?;
    file.read_exact(&mut certificate.lan_key)?;
    file.read_exact(&mut certificate.signature_key)?;
    file.read_exact(&mut certificate.alternate_signature_keys)?;

    while file.stream_position()? < start + certificate.size as u64 {
        certificate.reserved.push(file.read_u8()?);
    }

    Ok(certificate)
}

fn load_section_headers(file: &mut File, image_header: &ImageHeader) -> Result<Vec<SectionHeader>> {
    file.seek(SeekFrom::Start(
        (image_header.section_headers_address - image_header.base_address).into(),
    ))?;

    let mut headers = Vec::with_capacity(image_header.number_of_sections as usize);
    for _ in 0..image_header.number_of_sections {
        let mut h = SectionHeader::default();

        h.section_flags = file.read_u32::<LE>()?;
        h.virtual_address = file.read_u32::<LE>()?;
        h.virtual_size = file.read_u32::<LE>()?;
        h.raw_address = file.read_u32::<LE>()?;
        h.raw_size = file.read_u32::<LE>()?;
        h.section_name_address = file.read_u32::<LE>()?;
        h.section_name_reference_count = file.read_u32::<LE>()?;
        h.head_shared_page_reference_count_address = file.read_u32::<LE>()?;
        h.tail_shared_page_reference_count_address = file.read_u32::<LE>()?;
        file.read_exact(&mut h.section_digest)?;

        headers.push(h);
    }

    Ok(headers)
}

fn load_section_names(
    file: &mut File,
    image_header: &ImageHeader,
    sections_headers: &Vec<SectionHeader>,
) -> Result<Vec<String>> {
    let mut strings = vec![];

    for hdr in sections_headers.iter() {
        file.seek(SeekFrom::Start(
            (hdr.section_name_address - image_header.base_address) as u64,
        ))?;

        // Read null-terminated string
        let mut string = vec![];
        loop {
            let c = file.read_u8()?;
            string.push(c);
            if c == b'\0' {
                break;
            }
        }
        strings.push(String::from_utf8(string).expect("Section name not valid"));
    }

    Ok(strings)
}

fn load_library_versions(
    file: &mut File,
    image_header: &ImageHeader,
) -> Result<Vec<LibraryVersion>> {
    file.seek(SeekFrom::Start(
        (image_header.library_versions_address - image_header.base_address).into(),
    ))?;

    let mut library_versions = Vec::with_capacity(image_header.number_of_library_versions as usize);
    for _ in 0..image_header.number_of_library_versions {
        let mut l = LibraryVersion::default();

        file.read_exact(&mut l.library_name)?;
        l.major_version = file.read_u16::<LE>()?;
        l.minor_version = file.read_u16::<LE>()?;
        l.build_version = file.read_u16::<LE>()?;
        l.library_flags = file.read_u16::<LE>()?;

        library_versions.push(l);
    }

    Ok(library_versions)
}

fn load_debug_filename(file: &mut File, image_header: &ImageHeader) -> Result<String> {
    file.seek(SeekFrom::Start(
        (image_header.debug_filename_address - image_header.base_address) as u64,
    ))?;

    // Read null-terminated string
    let mut string = vec![];
    loop {
        let c = file.read_u8()?;
        string.push(c);
        if c == b'\0' {
            break;
        }
    }
    Ok(String::from_utf8(string).unwrap())
}

fn load_debug_pathname(file: &mut File, image_header: &ImageHeader) -> Result<String> {
    file.seek(SeekFrom::Start(
        (image_header.debug_pathname_address - image_header.base_address) as u64,
    ))?;

    // Read null-terminated string
    let mut string = vec![];
    loop {
        let c = file.read_u8()?;
        string.push(c);
        if c == b'\0' {
            break;
        }
    }
    Ok(String::from_utf8(string).unwrap())
}

fn load_debug_unicode_filename(file: &mut File, image_header: &ImageHeader) -> Result<Vec<u16>> {
    file.seek(SeekFrom::Start(
        (image_header.debug_unicode_filename_address - image_header.base_address) as u64,
    ))?;

    // Read null-terminated string
    let mut string = vec![];
    loop {
        let c = file.read_u16::<LE>()?;
        string.push(c);
        if c == 0 {
            break;
        }
    }
    Ok(string)
}

fn load_logo_bitmap(file: &mut File, image_header: &ImageHeader) -> Result<LogoBitmap> {
    file.seek(SeekFrom::Start(
        (image_header.logo_bitmap_address - image_header.base_address).into(),
    ))?;

    let mut buf = vec![0u8; image_header.logo_bitmap_size as usize];
    file.read_exact(&mut buf)?;
    Ok(LogoBitmap { bitmap: buf })
}

fn load_section(file: &mut File, section_header: &SectionHeader) -> Result<Section> {
    file.seek(SeekFrom::Start(section_header.raw_address as u64))?;
    let mut section = Section::default();

    let mut buf = vec![0u8; section_header.raw_size as usize];
    file.read_exact(&mut buf)?;
    section.bytes = buf;

    Ok(section)
}

/// This is a testing function to learn the format
/// Adding extra header padding expands into the beginning of section virtual memory
/// So this crashes the system somewhere beyond 0x800 added bytes (and likely corrupts
/// game memory somewhere before that)
pub fn add_padding_bytes(num_bytes: u32, xbe: &XBE) -> Result<()> {
    std::fs::copy("baserom/default.xbe", "output/default.xbe")?;

    {
        let mut output = OpenOptions::new().write(true).open("output/default.xbe")?;
        output.seek(SeekFrom::Current(0x108))?;
        output.write_u32::<LE>(xbe.image_header.size_of_headers + num_bytes)?;
        output.seek(SeekFrom::Current(0xC))?;
        output.write_u32::<LE>(xbe.image_header.certificate_address + num_bytes)?;
        output.seek(SeekFrom::Current(4))?;
        output.write_u32::<LE>(xbe.image_header.section_headers_address + num_bytes)?;

        output.seek(SeekFrom::Current(0x28))?;
        output.write_u32::<LE>(xbe.image_header.debug_pathname_address + num_bytes)?;
        output.write_u32::<LE>(xbe.image_header.debug_filename_address + num_bytes)?;
        output.write_u32::<LE>(xbe.image_header.debug_unicode_filename_address + num_bytes)?;

        output.seek(SeekFrom::Current(0x10))?;
        output.write_u32::<LE>(xbe.image_header.library_versions_address + num_bytes)?;
        output.write_u32::<LE>(xbe.image_header.kernel_library_version_address + num_bytes)?;
        output.write_u32::<LE>(xbe.image_header.xapi_library_version_address + num_bytes)?;
        output.write_u32::<LE>(xbe.image_header.logo_bitmap_address + num_bytes)?;

        output.seek(SeekFrom::Current(4))?;
        let buf = vec![0u8; num_bytes as usize];
        output.write(&buf)?;
    }

    let rest = std::fs::read("baserom/default.xbe")?;

    let mut output = std::fs::OpenOptions::new()
        .write(true)
        .open("output/default.xbe")?;
    output.seek(SeekFrom::Current(0x178 + (num_bytes as i64)))?;

    output.write(&rest[0x178..])?;

    for i in 0..xbe.image_header.number_of_sections {
        output.seek(SeekFrom::Start(
            (xbe.image_header.section_headers_address - xbe.image_header.base_address
                + num_bytes
                + (i * 0x38)
                + 0xC)
                .into(),
        ))?;
        output.write_u32::<LE>(xbe.section_headers[i as usize].raw_address + num_bytes)?;
        output.seek(SeekFrom::Current(4))?;
        output.write_u32::<LE>(xbe.section_headers[i as usize].section_name_address + num_bytes)?;
    }

    Ok(())
}

pub fn add_test_section(xbe: &XBE) -> Result<()> {
    // const size: u32 = 0x10;
    // const section: &[u8] = b"0123456789ABCDEF";

    // std::fs::copy("baserom/default.xbe", "output/default.xbe")?;
    // let mut f = OpenOptions::new()
    //     .read(true)
    //     .write(true)
    //     .open("output/default.xbe")?;

    // let mut header_buf = Cursor::new(vec![0u8; 0x1000]);
    // f.read(&mut header_buf.get_mut()[0..0x1c0])?;

    // // Update image size
    // header_buf.set_position(0x108);
    // header_buf.write_u32::<LE>(xbe.image_header.size_of_headers + 0x38)?;
    // header_buf.write_u32::<LE>(xbe.image_header.size_of_image + size)?;

    // // Update section count
    // header_buf.set_position(0x11C);
    // header_buf.write_u32::<LE>(xbe.image_header.number_of_sections + 1)?;

    // // Update debug path addresses
    // header_buf.set_position(0x14C);
    // header_buf.write_u32::<LE>(xbe.image_header.debug_pathname_address + 0x30)?;
    // header_buf.write_u32::<LE>(xbe.image_header.debug_filename_address + 0x30)?;
    // header_buf.write_u32::<LE>(xbe.image_header.debug_unicode_filename_address + 0x30)?;

    // // Update library versions address
    // header_buf.set_position(0x164);
    // header_buf.write_u32::<LE>(xbe.image_header.library_versions_address + 0x30)?;

    // // Update logo bitmap address
    // header_buf.set_position(0x170);
    // header_buf.write_u32::<LE>(xbe.image_header.logo_bitmap_address + 0x30)?;

    ////////////

    // // Update image size
    // f.seek(SeekFrom::Current(0x108))?;
    // f.write_u32::<LittleEndian>(xbe.image_header.size_of_headers + 0x38)?;
    // f.write_u32::<LittleEndian>(xbe.image_header.size_of_image + size)?;

    // // Update section count
    // f.seek(SeekFrom::Current(0xC))?;
    // f.write_u32::<LittleEndian>(xbe.image_header.number_of_sections + 1)?;

    // // Update debug path addresses
    // f.seek(SeekFrom::Current(0x2C))?;
    // f.write_u32::<LittleEndian>(xbe.image_header.debug_pathname_address + 0x30)?;
    // f.write_u32::<LittleEndian>(xbe.image_header.debug_filename_address + 0x30)?;
    // f.write_u32::<LittleEndian>(xbe.image_header.debug_unicode_filename_address + 0x30)?;

    // // Update library versions address
    // f.seek(SeekFrom::Current(0xC))?;
    // f.write_u32::<LittleEndian>(xbe.image_header.library_versions_address + 0x30)?;

    // // Update logo bitmap address
    // f.seek(SeekFrom::Current(0x8))?;
    // f.write_u32::<LittleEndian>(xbe.image_header.logo_bitmap_address + 0x30)?;

    // // Update section addresses
    // for i in 0..xbe.image_header.number_of_sections {
    //     f.seek(SeekFrom::Start(
    //         (xbe.image_header.section_headers_address - xbe.image_header.base_address
    //             + 0x30
    //             + (i * 0x38)
    //             + 0xC)
    //             .into(),
    //     ))?;
    //     f.write_u32::<LittleEndian>(xbe.section_headers[i as usize].raw_address + 0x30)?;
    //     f.seek(SeekFrom::Current(4))?;
    //     f.write_u32::<LittleEndian>(xbe.section_headers[i as usize].section_name_address + 0x30)?;
    // }
    // let last_section = &xbe.section_headers[xbe.image_header.number_of_sections as usize - 1];

    // // Find end of header names
    // let mut b = vec![0u8];
    // f.seek(SeekFrom::Start(last_section.section_name_address as u64))?;
    // let mut len = 0;
    // loop {
    //     f.read(&mut b)?;
    //     len += 1;
    //     if b[0] == b'\0' {
    //         break;
    //     }
    // }

    // // Add section header
    // f.seek(SeekFrom::Start(
    //     (xbe.image_header.section_headers_address - xbe.image_header.base_address) as u64
    //         + (xbe.image_header.number_of_sections * 0x38) as u64,
    // ))?;

    // f.write_u32::<LittleEndian>(2)?;

    // let virtual_address = last_section.virtual_address + last_section.virtual_size;
    // f.write_u32::<LittleEndian>(virtual_address)?;
    // f.write_u32::<LittleEndian>(size)?;

    // let raw_address = last_section.raw_address + last_section.raw_size;
    // f.write_u32::<LittleEndian>(raw_address)?;
    // f.write_u32::<LittleEndian>(size)?;

    // f.write_u32::<LittleEndian>(last_section.section_name_address + len)?;
    // f.write_u32::<LittleEndian>(0)?;
    // f.write_u32::<LittleEndian>(last_section.tail_shared_page_reference_count_address)?;
    // f.write_u32::<LittleEndian>(last_section.tail_shared_page_reference_count_address + 2)?;

    // let section_digest = [0u8; 0x14];
    // f.write(&section_digest)?;

    Ok(())
}
