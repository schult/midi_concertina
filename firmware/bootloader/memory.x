MEMORY
{
    FLASH            : ORIGIN = 0x08000000, LENGTH = 8K
    BOOTLOADER_STATE : ORIGIN = 0x08002000, LENGTH = 0x1D00
    ACTIVE           : ORIGIN = 0x08003D00, LENGTH = 0xE100
    DFU              : ORIGIN = 0x08011E00, LENGTH = 0xE180

    RAM              : ORIGIN = 0x20000000, LENGTH = 20K
}

__bootloader_state_start = ORIGIN(BOOTLOADER_STATE) - ORIGIN(FLASH);
__bootloader_state_end = ORIGIN(BOOTLOADER_STATE) + LENGTH(BOOTLOADER_STATE) - ORIGIN(FLASH);

__bootloader_active_start = ORIGIN(ACTIVE) - ORIGIN(FLASH);
__bootloader_active_end = ORIGIN(ACTIVE) + LENGTH(ACTIVE) - ORIGIN(FLASH);

__bootloader_dfu_start = ORIGIN(DFU) - ORIGIN(FLASH);
__bootloader_dfu_end = ORIGIN(DFU) + LENGTH(DFU) - ORIGIN(FLASH);
