#[cfg(feature = "verbose-boot")]
#[derive(Clone, Copy)]
pub struct CpuInfo {
    pub vendor: [u8; 12],
    pub max_basic_leaf: u32,

    pub has_apic: bool,
    pub has_x2apic: bool,
    pub has_tsc: bool,
    pub has_msr: bool,
    pub has_sse: bool,
    pub has_sse2: bool,
    pub has_xsave: bool,
}

#[cfg(feature = "verbose-boot")]
impl CpuInfo {
    pub fn detect() -> Self {
        let (max_basic_leaf, ebx, ecx, edx) =
            cpuid(0);

        let mut vendor = [0u8; 12];

        vendor[0..4]
            .copy_from_slice(&ebx.to_le_bytes());

        vendor[4..8]
            .copy_from_slice(&edx.to_le_bytes());

        vendor[8..12]
            .copy_from_slice(&ecx.to_le_bytes());

        let (feature_ecx, feature_edx) =
            if max_basic_leaf >= 1 {
                let (_, _, ecx, edx) =
                    cpuid(1);

                (ecx, edx)
            } else {
                (0, 0)
            };

        let has_apic =
            (feature_edx & (1 << 9)) != 0;

        let has_tsc =
            (feature_edx & (1 << 4)) != 0;

        let has_msr =
            (feature_edx & (1 << 5)) != 0;

        let has_sse =
            (feature_edx & (1 << 25)) != 0;

        let has_sse2 =
            (feature_edx & (1 << 26)) != 0;

        let has_xsave =
            (feature_ecx & (1 << 26)) != 0;

        let has_x2apic =
            (feature_ecx & (1 << 21)) != 0;

        Self {
            vendor,
            max_basic_leaf,

            has_apic,
            has_x2apic,
            has_tsc,
            has_msr,
            has_sse,
            has_sse2,
            has_xsave,
        }
    }
}

#[cfg(feature = "verbose-boot")]
#[inline]
fn cpuid(leaf: u32) -> (u32, u32, u32, u32) {
    let result =
        core::arch::x86_64::__cpuid(leaf);

    (
        result.eax,
        result.ebx,
        result.ecx,
        result.edx,
    )
}