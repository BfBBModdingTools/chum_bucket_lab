use std::{
    fs::File,
    io::{Read, Result, Seek, SeekFrom, Write},
};

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

#[derive(Default, Debug)]
pub struct XBE {
    pub image_header: ImageHeader,
    pub certificate: Certificate,
    pub section_headers: Vec<SectionHeader>,
    pub library_version: Vec<LibraryVersion>,
}

#[derive(Debug)]
pub struct ImageHeader {
    pub magic_number: [u8; 4],
    pub digital_signature: [u8; 256],
    pub base_address: u32,
    pub size_of_headers: u32,
    pub size_of_image: u32,
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
    pub lan_key: [u8; 16],
    pub signature_key: [u8; 16],
    pub alternate_signature_keys: [u8; 16],
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
            alternate_signature_keys: [0u8; 16],
        }
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

#[derive(Debug, Default)]
pub struct LibraryVersion {
    pub library_name: [u8; 8],
    pub major_version: u16,
    pub minor_version: u16,
    pub build_version: u16,
    pub library_flags: u16,
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

pub fn load_xbe(mut file: File) -> std::io::Result<XBE> {
    // let mut xbe = XBE::default();

    // Read header data
    let image_header = load_image_header(&mut file)?;

    // Read certificate data
    let certificate = load_certificate(&mut file, &image_header)?;

    // Read section headers
    let section_headers = load_section_headers(&mut file, &image_header)?;

    // Read library versions
    let library_version = load_library_versions(&mut file, &image_header)?;
    Ok(XBE {
        image_header,
        certificate,
        section_headers,
        library_version,
    })
}

fn load_image_header(file: &mut File) -> Result<ImageHeader> {
    let mut header = ImageHeader::default();

    file.read_exact(&mut header.magic_number)?;
    file.read_exact(&mut header.digital_signature)?;
    header.base_address = file.read_u32::<LittleEndian>()?;
    header.size_of_headers = file.read_u32::<LittleEndian>()?;
    header.size_of_image = file.read_u32::<LittleEndian>()?;
    header.size_of_image_header = file.read_u32::<LittleEndian>()?;
    header.time_date = file.read_u32::<LittleEndian>()?;
    header.certificate_address = file.read_u32::<LittleEndian>()?;
    header.number_of_sections = file.read_u32::<LittleEndian>()?;
    header.section_headers_address = file.read_u32::<LittleEndian>()?;
    header.initialization_flags = file.read_u32::<LittleEndian>()?;
    header.entry_point = file.read_u32::<LittleEndian>()?;
    header.tls_address = file.read_u32::<LittleEndian>()?;
    header.pe_stack_commit = file.read_u32::<LittleEndian>()?;
    header.pe_heap_reserve = file.read_u32::<LittleEndian>()?;
    header.pe_head_commit = file.read_u32::<LittleEndian>()?;
    header.pe_base_address = file.read_u32::<LittleEndian>()?;
    header.pe_size_of_image = file.read_u32::<LittleEndian>()?;
    header.pe_checksum = file.read_u32::<LittleEndian>()?;
    header.pe_time_date = file.read_u32::<LittleEndian>()?;
    header.debug_pathname_address = file.read_u32::<LittleEndian>()?;
    header.debug_filename_address = file.read_u32::<LittleEndian>()?;
    header.debug_unicode_filename_address = file.read_u32::<LittleEndian>()?;
    header.kernel_image_thunk_address = file.read_u32::<LittleEndian>()?;
    header.non_kernel_import_directory_address = file.read_u32::<LittleEndian>()?;
    header.number_of_library_versions = file.read_u32::<LittleEndian>()?;
    header.library_versions_address = file.read_u32::<LittleEndian>()?;
    header.kernel_library_version_address = file.read_u32::<LittleEndian>()?;
    header.xapi_library_version_address = file.read_u32::<LittleEndian>()?;
    header.logo_bitmap_address = file.read_u32::<LittleEndian>()?;
    header.logo_bitmap_size = file.read_u32::<LittleEndian>()?;
    Ok(header)
}

fn load_certificate(file: &mut File, header: &ImageHeader) -> Result<Certificate> {
    file.seek(SeekFrom::Start(
        (header.certificate_address - header.base_address).into(),
    ))?;

    let mut certificate = Certificate::default();

    certificate.size = file.read_u32::<LittleEndian>()?;
    certificate.time_date = file.read_u32::<LittleEndian>()?;
    certificate.title_id = file.read_u32::<LittleEndian>()?;
    file.read_exact(&mut certificate.title_name)?;
    file.read_exact(&mut certificate.alternate_title_ids)?;
    certificate.allowed_media = file.read_u32::<LittleEndian>()?;
    certificate.game_region = file.read_u32::<LittleEndian>()?;
    certificate.game_ratings = file.read_u32::<LittleEndian>()?;
    certificate.disk_number = file.read_u32::<LittleEndian>()?;
    certificate.version = file.read_u32::<LittleEndian>()?;
    file.read_exact(&mut certificate.lan_key)?;
    file.read_exact(&mut certificate.signature_key)?;
    file.read_exact(&mut certificate.alternate_signature_keys)?;
    Ok(certificate)
}

fn load_section_headers(file: &mut File, image_header: &ImageHeader) -> Result<Vec<SectionHeader>> {
    file.seek(SeekFrom::Start(
        (image_header.section_headers_address - image_header.base_address).into(),
    ))?;

    let mut headers = Vec::with_capacity(image_header.number_of_sections as usize);
    for _ in 0..image_header.number_of_sections {
        // file.seek(SeekFrom::Start((addr - image_header.base_address).into()))?;

        let mut h = SectionHeader::default();

        h.section_flags = file.read_u32::<LittleEndian>()?;
        h.virtual_address = file.read_u32::<LittleEndian>()?;
        h.virtual_size = file.read_u32::<LittleEndian>()?;
        h.raw_address = file.read_u32::<LittleEndian>()?;
        h.raw_size = file.read_u32::<LittleEndian>()?;
        h.section_name_address = file.read_u32::<LittleEndian>()?;
        h.section_name_reference_count = file.read_u32::<LittleEndian>()?;
        h.head_shared_page_reference_count_address = file.read_u32::<LittleEndian>()?;
        h.tail_shared_page_reference_count_address = file.read_u32::<LittleEndian>()?;
        file.read_exact(&mut h.section_digest)?;

        headers.push(h);
    }

    Ok(headers)
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
        l.major_version = file.read_u16::<LittleEndian>()?;
        l.minor_version = file.read_u16::<LittleEndian>()?;
        l.build_version = file.read_u16::<LittleEndian>()?;
        l.library_flags = file.read_u16::<LittleEndian>()?;

        library_versions.push(l);
    }

    Ok(library_versions)
}

/// This is a testing function to learn the format
/// Adding extra header padding expands into the beginning of section virtual memory
/// So this crashes the system somewhere beyond 0x800 added bytes (and likely corrupts
/// game memory somewhere before that)
pub fn add_padding_bytes(num_bytes: u32, xbe: &XBE) -> Result<()> {
    std::fs::copy("baserom/default.xbe", "output/default.xbe")?;

    {
        let mut output = std::fs::OpenOptions::new()
            .write(true)
            .open("output/default.xbe")?;
        output.seek(SeekFrom::Current(0x108))?;
        output.write_u32::<LittleEndian>(xbe.image_header.size_of_headers + num_bytes)?;
        output.seek(SeekFrom::Current(0xC))?;
        output.write_u32::<LittleEndian>(xbe.image_header.certificate_address + num_bytes)?;
        output.seek(SeekFrom::Current(4))?;
        output.write_u32::<LittleEndian>(xbe.image_header.section_headers_address + num_bytes)?;

        output.seek(SeekFrom::Current(0x28))?;
        output.write_u32::<LittleEndian>(xbe.image_header.debug_pathname_address + num_bytes)?;
        output.write_u32::<LittleEndian>(xbe.image_header.debug_filename_address + num_bytes)?;
        output.write_u32::<LittleEndian>(
            xbe.image_header.debug_unicode_filename_address + num_bytes,
        )?;

        output.seek(SeekFrom::Current(0x10))?;
        output.write_u32::<LittleEndian>(xbe.image_header.library_versions_address + num_bytes)?;
        output.write_u32::<LittleEndian>(
            xbe.image_header.kernel_library_version_address + num_bytes,
        )?;
        output
            .write_u32::<LittleEndian>(xbe.image_header.xapi_library_version_address + num_bytes)?;
        output.write_u32::<LittleEndian>(xbe.image_header.logo_bitmap_address + num_bytes)?;

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
        output
            .write_u32::<LittleEndian>(xbe.section_headers[i as usize].raw_address + num_bytes)?;
        output.seek(SeekFrom::Current(4))?;
        output.write_u32::<LittleEndian>(
            xbe.section_headers[i as usize].section_name_address + num_bytes,
        )?;
        // output.seek(SeekFrom::Current(4))?;
        // output.write_u32::<LittleEndian>(
        //     xbe.section_headers[i as usize].head_shared_page_reference_count_address + num_bytes,
        // )?;
        // output.write_u32::<LittleEndian>(
        //     xbe.section_headers[i as usize].tail_shared_page_reference_count_address + num_bytes,
        // )?;
    }

    Ok(())
}
