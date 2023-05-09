use crate::bus::cartridge::mappers::{BankSizeKb, ChrType, NametableMirroring};
use crate::bus::cartridge::MapperImpl;
use bincode::{Decode, Encode};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Encode, Decode)]
enum PrgMode {
    Mode0,
    Mode1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Encode, Decode)]
enum ChrMode {
    Mode0,
    Mode1,
}

#[derive(Debug, Clone, Encode, Decode)]
struct BankMapping {
    prg_mode: PrgMode,
    chr_mode: ChrMode,
    prg_rom_len: u32,
    chr_len: u32,
    prg_bank_0: u8,
    prg_bank_1: u8,
    chr_banks: [u8; 6],
}

impl BankMapping {
    fn new(prg_rom_len: u32, chr_len: u32) -> Self {
        Self {
            prg_mode: PrgMode::Mode0,
            chr_mode: ChrMode::Mode0,
            prg_rom_len,
            chr_len,
            prg_bank_0: 0,
            prg_bank_1: 0,
            chr_banks: [0; 6],
        }
    }

    fn map_prg_rom_address(&self, address: u16) -> u32 {
        match (self.prg_mode, address) {
            (_, 0x0000..=0x7FFF) => panic!("invalid MMC3 PRG ROM address: 0x{address:04X}"),
            (PrgMode::Mode0, 0x8000..=0x9FFF) | (PrgMode::Mode1, 0xC000..=0xDFFF) => {
                BankSizeKb::Eight.to_absolute_address(self.prg_bank_0, address)
            }
            (_, 0xA000..=0xBFFF) => BankSizeKb::Eight.to_absolute_address(self.prg_bank_1, address),
            (PrgMode::Mode0, 0xC000..=0xDFFF) | (PrgMode::Mode1, 0x8000..=0x9FFF) => {
                // Fixed at second-to-last bank
                BankSizeKb::Eight.to_absolute_address_from_end(2_u32, self.prg_rom_len, address)
            }
            (_, 0xE000..=0xFFFF) => {
                // Fixed at last bank
                BankSizeKb::Eight.to_absolute_address_last_bank(self.prg_rom_len, address)
            }
        }
    }

    fn map_pattern_table_address(&self, address: u16) -> u32 {
        let mapped_address = match (self.chr_mode, address) {
            // 2KB banks are treated as 1KB bank numbers while ignoring the lowest bit
            (ChrMode::Mode0, 0x0000..=0x07FF) | (ChrMode::Mode1, 0x1000..=0x17FF) => {
                BankSizeKb::Two.to_absolute_address(self.chr_banks[0] >> 1, address)
            }
            (ChrMode::Mode0, 0x0800..=0x0FFF) | (ChrMode::Mode1, 0x1800..=0x1FFF) => {
                BankSizeKb::Two.to_absolute_address(self.chr_banks[1] >> 1, address)
            }
            (ChrMode::Mode0, 0x1000..=0x13FF) | (ChrMode::Mode1, 0x0000..=0x03FF) => {
                BankSizeKb::One.to_absolute_address(self.chr_banks[2], address)
            }
            (ChrMode::Mode0, 0x1400..=0x17FF) | (ChrMode::Mode1, 0x0400..=0x07FF) => {
                BankSizeKb::One.to_absolute_address(self.chr_banks[3], address)
            }
            (ChrMode::Mode0, 0x1800..=0x1BFF) | (ChrMode::Mode1, 0x0800..=0x0BFF) => {
                BankSizeKb::One.to_absolute_address(self.chr_banks[4], address)
            }
            (ChrMode::Mode0, 0x1C00..=0x1FFF) | (ChrMode::Mode1, 0x0C00..=0x0FFF) => {
                BankSizeKb::One.to_absolute_address(self.chr_banks[5], address)
            }
            (_, 0x2000..=0xFFFF) => {
                panic!("invalid MMC3 CHR pattern table address: 0x{address:04X}")
            }
        };
        mapped_address & (self.chr_len - 1)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Encode, Decode)]
enum BankUpdate {
    PrgBank0,
    PrgBank1,
    ChrBank(u8),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Encode, Decode)]
enum Variant {
    Mmc3,
    Mmc6,
    McAcc,
}

impl Variant {
    fn name(self) -> &'static str {
        match self {
            Self::Mmc3 => "MMC3",
            Self::Mmc6 => "MMC6",
            Self::McAcc => "MMC3 (MC-ACC variant)",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Encode, Decode)]
enum RamMode {
    Mmc3Enabled,
    Mmc3WritesDisabled,
    Mmc6Enabled {
        first_half_reads: bool,
        first_half_writes: bool,
        second_half_reads: bool,
        second_half_writes: bool,
    },
    Disabled,
}

impl RamMode {
    fn reads_enabled(self, address: u16) -> bool {
        match self {
            Self::Mmc3Enabled | Self::Mmc3WritesDisabled => true,
            Self::Mmc6Enabled {
                first_half_reads,
                second_half_reads,
                ..
            } => {
                if address & 0x0200 != 0 {
                    second_half_reads
                } else {
                    first_half_reads
                }
            }
            Self::Disabled => false,
        }
    }

    fn writes_enabled(self, address: u16) -> bool {
        match self {
            Self::Mmc3Enabled => true,
            Self::Mmc6Enabled {
                first_half_writes,
                second_half_writes,
                ..
            } => {
                if address & 0x0200 != 0 {
                    second_half_writes
                } else {
                    first_half_writes
                }
            }
            Self::Disabled | Self::Mmc3WritesDisabled => false,
        }
    }
}

#[derive(Debug, Clone, Encode, Decode)]
enum Mmc3NametableMirroring {
    Standard(NametableMirroring),
    FourScreenVram { external_vram: Box<[u8; 4096]> },
}

#[derive(Debug, Clone, Encode, Decode)]
pub(crate) struct Mmc3 {
    variant: Variant,
    chr_type: ChrType,
    bank_mapping: BankMapping,
    nametable_mirroring: Mmc3NametableMirroring,
    bank_update_select: BankUpdate,
    ram_mode: RamMode,
    interrupt_flag: bool,
    irq_counter: u8,
    irq_reload_value: u8,
    irq_reload_flag: bool,
    irq_enabled: bool,
    last_a12_read: u16,
    a12_low_cycles: u32,
    mc_acc_pulse_counter: u8,
}

const ACC_COUNTER_INIT_VALUE: u8 = 6;

impl Mmc3 {
    pub(crate) fn new(
        chr_type: ChrType,
        prg_rom_len: u32,
        chr_size: u32,
        sub_mapper_number: u8,
        has_four_screen_vram: bool,
    ) -> Self {
        let variant = match sub_mapper_number {
            1 => Variant::Mmc6,
            3 => Variant::McAcc,
            _ => Variant::Mmc3,
        };
        Self {
            variant,
            chr_type,
            bank_mapping: BankMapping::new(prg_rom_len, chr_size),
            nametable_mirroring: if has_four_screen_vram {
                Mmc3NametableMirroring::FourScreenVram {
                    external_vram: Box::new([0; 4096]),
                }
            } else {
                Mmc3NametableMirroring::Standard(NametableMirroring::Vertical)
            },
            bank_update_select: BankUpdate::ChrBank(0),
            ram_mode: RamMode::Disabled,
            interrupt_flag: false,
            irq_counter: 0,
            irq_reload_value: 0,
            irq_reload_flag: false,
            irq_enabled: false,
            last_a12_read: 0,
            a12_low_cycles: 0,
            mc_acc_pulse_counter: ACC_COUNTER_INIT_VALUE,
        }
    }
}

impl MapperImpl<Mmc3> {
    pub(crate) fn read_cpu_address(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x401F => panic!("invalid CPU map address: 0x{address:04X}"),
            0x4020..=0x5FFF => 0xFF,
            0x6000..=0x7FFF => {
                if self.data.ram_mode.reads_enabled(address) && !self.cartridge.prg_ram.is_empty() {
                    self.cartridge.get_prg_ram(u32::from(address & 0x1FFF))
                } else {
                    0xFF
                }
            }
            0x8000..=0xFFFF => self
                .cartridge
                .get_prg_rom(self.data.bank_mapping.map_prg_rom_address(address)),
        }
    }

    pub(crate) fn write_cpu_address(&mut self, address: u16, value: u8) {
        match address {
            0x0000..=0x401F => panic!("invalid CPU map address: 0x{address:04X}"),
            0x4020..=0x5FFF => {}
            0x6000..=0x7FFF => {
                if self.data.ram_mode.writes_enabled(address) && !self.cartridge.prg_ram.is_empty()
                {
                    self.cartridge
                        .set_prg_ram(u32::from(address & 0x1FFF), value);
                }
            }
            0x8000..=0x9FFF => {
                if address & 0x01 == 0 {
                    self.data.bank_mapping.chr_mode = if value & 0x80 != 0 {
                        ChrMode::Mode1
                    } else {
                        ChrMode::Mode0
                    };
                    self.data.bank_mapping.prg_mode = if value & 0x40 != 0 {
                        PrgMode::Mode1
                    } else {
                        PrgMode::Mode0
                    };
                    self.data.bank_update_select = match value & 0x07 {
                        masked_value @ 0x00..=0x05 => BankUpdate::ChrBank(masked_value),
                        0x06 => BankUpdate::PrgBank0,
                        0x07 => BankUpdate::PrgBank1,
                        _ => unreachable!(
                            "masking with 0x07 should always be in the range 0x00..=0x07"
                        ),
                    };

                    if self.data.variant == Variant::Mmc6 {
                        let ram_enabled = value & 0x20 != 0;
                        if !ram_enabled {
                            self.data.ram_mode = RamMode::Disabled;
                        } else if ram_enabled && self.data.ram_mode == RamMode::Disabled {
                            self.data.ram_mode = RamMode::Mmc6Enabled {
                                first_half_reads: false,
                                first_half_writes: false,
                                second_half_reads: false,
                                second_half_writes: false,
                            };
                        }
                    }
                } else {
                    match self.data.bank_update_select {
                        BankUpdate::ChrBank(chr_bank) => {
                            self.data.bank_mapping.chr_banks[chr_bank as usize] = value;
                        }
                        BankUpdate::PrgBank0 => {
                            self.data.bank_mapping.prg_bank_0 = value;
                        }
                        BankUpdate::PrgBank1 => {
                            self.data.bank_mapping.prg_bank_1 = value;
                        }
                    }
                }
            }
            0xA000..=0xBFFF => {
                if address & 0x01 == 0
                    && matches!(
                        self.data.nametable_mirroring,
                        Mmc3NametableMirroring::Standard(..)
                    )
                {
                    let nametable_mirroring = if value & 0x01 != 0 {
                        NametableMirroring::Horizontal
                    } else {
                        NametableMirroring::Vertical
                    };
                    self.data.nametable_mirroring =
                        Mmc3NametableMirroring::Standard(nametable_mirroring);
                } else {
                    match self.data.variant {
                        Variant::Mmc6 => {
                            self.data.ram_mode = if self.data.ram_mode == RamMode::Disabled {
                                // $A001 writes are ignored if RAM is disabled via $8000
                                RamMode::Disabled
                            } else {
                                let first_half_writes = value & 0x10 != 0;
                                let first_half_reads = value & 0x20 != 0;
                                let second_half_writes = value & 0x40 != 0;
                                let second_half_reads = value & 0x80 != 0;
                                RamMode::Mmc6Enabled {
                                    first_half_reads,
                                    first_half_writes,
                                    second_half_reads,
                                    second_half_writes,
                                }
                            };
                        }
                        Variant::Mmc3 | Variant::McAcc => {
                            self.data.ram_mode = if value & 0x80 == 0 {
                                RamMode::Disabled
                            } else if value & 0x40 != 0 {
                                RamMode::Mmc3WritesDisabled
                            } else {
                                RamMode::Mmc3Enabled
                            };
                        }
                    }
                }
            }
            0xC000..=0xDFFF => {
                if address & 0x01 == 0 {
                    self.data.irq_reload_value = value;
                } else {
                    self.data.irq_reload_flag = true;
                    self.data.mc_acc_pulse_counter = ACC_COUNTER_INIT_VALUE;
                }
            }
            0xE000..=0xFFFF => {
                if address & 0x01 == 0 {
                    self.data.irq_enabled = false;
                    self.data.interrupt_flag = false;
                } else {
                    self.data.irq_enabled = true;
                }
            }
        }
    }

    fn clock_irq(&mut self) {
        log::trace!(
            "IRQ clocked; counter={}, reload_flag={}, reload_value={}",
            self.data.irq_counter,
            self.data.irq_reload_flag,
            self.data.irq_reload_value
        );

        if self.data.irq_counter == 0 || self.data.irq_reload_flag {
            self.data.irq_counter = self.data.irq_reload_value;
            self.data.irq_reload_flag = false;
        } else {
            self.data.irq_counter -= 1;
        }

        if self.data.irq_counter == 0 && self.data.irq_enabled {
            self.data.interrupt_flag = true;
        }
    }

    fn process_ppu_address(&mut self, address: u16) {
        log::trace!("PPU bus address: {address:04X}");

        let a12 = address & (1 << 12);

        match self.data.variant {
            Variant::Mmc3 | Variant::Mmc6 => {
                if a12 != 0 && self.data.last_a12_read == 0 && self.data.a12_low_cycles >= 10 {
                    self.clock_irq();
                }
            }
            Variant::McAcc => {
                if a12 == 0 && self.data.last_a12_read != 0 {
                    self.data.mc_acc_pulse_counter += 1;
                    if self.data.mc_acc_pulse_counter == 8 {
                        self.clock_irq();
                        self.data.mc_acc_pulse_counter = 0;
                    }
                }
            }
        }

        self.data.last_a12_read = a12;
    }

    pub(crate) fn read_ppu_address(&mut self, address: u16, vram: &[u8; 2048]) -> u8 {
        match address & 0x3FFF {
            0x0000..=0x1FFF => self
                .data
                .chr_type
                .to_map_result(self.data.bank_mapping.map_pattern_table_address(address))
                .read(&self.cartridge, vram),
            0x2000..=0x3EFF => match &self.data.nametable_mirroring {
                Mmc3NametableMirroring::Standard(nametable_mirroring) => {
                    vram[nametable_mirroring.map_to_vram(address) as usize]
                }
                Mmc3NametableMirroring::FourScreenVram { external_vram } => {
                    external_vram[(address & 0x0FFF) as usize]
                }
            },
            0x3F00..=0xFFFF => panic!("invalid PPU map address: 0x{address:04X}"),
        }
    }

    pub(crate) fn write_ppu_address(&mut self, address: u16, value: u8, vram: &mut [u8; 2048]) {
        self.process_ppu_address(address);

        match address & 0x3FFF {
            0x0000..=0x1FFF => {
                self.data
                    .chr_type
                    .to_map_result(self.data.bank_mapping.map_pattern_table_address(address))
                    .write(value, &mut self.cartridge, vram);
            }
            0x2000..=0x3EFF => match &mut self.data.nametable_mirroring {
                Mmc3NametableMirroring::Standard(nametable_mirroring) => {
                    vram[nametable_mirroring.map_to_vram(address) as usize] = value;
                }
                Mmc3NametableMirroring::FourScreenVram { external_vram } => {
                    external_vram[(address & 0x0FFF) as usize] = value;
                }
            },
            0x3F00..=0xFFFF => panic!("invalid PPU map address: 0x{address:04X}"),
        }
    }

    pub(crate) fn interrupt_flag(&self) -> bool {
        self.data.interrupt_flag
    }

    pub(crate) fn tick(&mut self, ppu_bus_address: u16) {
        self.process_ppu_address(ppu_bus_address);

        if self.data.last_a12_read == 0 {
            self.data.a12_low_cycles += 1;
        } else {
            self.data.a12_low_cycles = 0;
        }
    }

    pub(crate) fn name(&self) -> &'static str {
        self.data.variant.name()
    }
}
