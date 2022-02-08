MEMORY
{
  /* NOTE 1 K = 1 KiBi = 1024 bytes */
}

MEMORY
{
  /* NOTE 1 K = 1 KiBi = 1024 bytes */
  FLASH                             : ORIGIN = 0x00027000, LENGTH = 1024K - 0x27000 - 28K
  RAM                               : ORIGIN = 0x20020000, LENGTH = 128K
  BOOTLOADER_STATE                  : ORIGIN = 0x000ff000, LENGTH = 4K
}

SECTIONS
{
  .bootloader_state :
  {
    KEEP(*(SORT(.bootloader_state*)))
  } > BOOTSTATE
}
