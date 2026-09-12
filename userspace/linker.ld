ENTRY(_start)

PHDRS
{
    text PT_LOAD FLAGS(5);
    rodata PT_LOAD FLAGS(4);
    data PT_LOAD FLAGS(6);
}

SECTIONS
{
    . = 0x0000010000000000;

    .text : ALIGN(4K)
    {
        KEEP(*(.text._start))
        *(.text*)
    } :text

    .rodata : ALIGN(4K)
    {
        *(.rodata*)
    } :rodata

    .data : ALIGN(4K)
    {
        *(.data*)
    } :data

    .bss : ALIGN(4K)
    {
        *(COMMON)
        *(.bss*)
    } :data

    /DISCARD/ :
    {
        *(.eh_frame*)
        *(.note*)
    }
}