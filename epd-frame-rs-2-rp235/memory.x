/*
* The CYW43 Firmware and CLM data must be 4-byte aligned.
* These parts are not included into the RP Firmware to save time during reflashing.
*
* The layout is:
* 0x10000000 - FLASH1
* 0x101C0000 - FLASH2
* 0x10380000 - CYW43FW
* 0x103BE000 - CYW43CLM
* 0x103C0000 - CONFIG
*/
__firmware_section_length = 1792K;
__cyw43_firmware_section_length = 248K;
__cyw43_firmware_length = 230321;
__cyw43_clm_section_length = 8K;
__cyw43_clm_length = 4752;
__config_section_length = 256K;

MEMORY {
    FLASH : ORIGIN = 0x10000000, LENGTH = __firmware_section_length + __firmware_section_length
    FLASH2 : ORIGIN = ORIGIN(FLASH) + LENGTH(FLASH), LENGTH = 0
    CYW43FW : ORIGIN = ORIGIN(FLASH2) + LENGTH(FLASH2), LENGTH = __cyw43_firmware_section_length
    CYW43CLM : ORIGIN = ORIGIN(CYW43FW) + LENGTH(CYW43FW), LENGTH = __cyw43_clm_section_length
    CONFIG : ORIGIN = ORIGIN(CYW43CLM) + LENGTH(CYW43CLM), LENGTH = __config_section_length

    RAM : ORIGIN = 0x20000000, LENGTH = 512K
    /*
     * RAM banks 8 and 9 use a direct mapping. They can be used to have
     * memory areas dedicated for some specific job, improving predictability
     * of access times.
     * Example: Separate stacks for core0 and core1.
     */
    SRAM4 : ORIGIN = 0x20080000, LENGTH = 4K
    SRAM5 : ORIGIN = 0x20081000, LENGTH = 4K
}

__cyw43_firmware_start = ORIGIN(CYW43FW);
__cyw43_clm_start = ORIGIN(CYW43CLM);
/*
* Uploading CYW43 firmware:
* probe-rs download cyw43-firmware/cyw43439-firmware/43439A0.bin --binary-format bin --chip RP235x --base-address 0x10380000
* probe-rs download cyw43-firmware/cyw43439-firmware/43439A0_clm.bin --binary-format bin --chip RP235x --base-address 0x103BE000
*/

__config_start = ORIGIN(CONFIG);
__config_length = LENGTH(CONFIG);

SECTIONS {
    /* ### Boot ROM info
     *
     * Goes after .vector_table, to keep it in the first 4K of flash
     * where the Boot ROM (and picotool) can find it
     */
    .start_block : ALIGN(4)
    {
        __start_block_addr = .;
        KEEP(*(.start_block));
        KEEP(*(.boot_info));
    } > FLASH

} INSERT AFTER .vector_table;

/* move .text to start /after/ the boot info */
_stext = ADDR(.start_block) + SIZEOF(.start_block);

SECTIONS {
    /* ### Picotool 'Binary Info' Entries
     *
     * Picotool looks through this block (as we have pointers to it in our
     * header) to find interesting information.
     */
    .bi_entries : ALIGN(4)
    {
        /* We put this in the header */
        __bi_entries_start = .;
        /* Here are the entries */
        KEEP(*(.bi_entries));
        /* Keep this block a nice round size */
        . = ALIGN(4);
        /* We put this in the header */
        __bi_entries_end = .;
    } > FLASH
} INSERT AFTER .text;

SECTIONS {
    /* ### Boot ROM extra info
     *
     * Goes after everything in our program, so it can contain a signature.
     */
    .end_block : ALIGN(4)
    {
        __end_block_addr = .;
        KEEP(*(.end_block));
    } > FLASH

} INSERT AFTER .uninit;

PROVIDE(start_to_end = __end_block_addr - __start_block_addr);
PROVIDE(end_to_start = __start_block_addr - __end_block_addr);
