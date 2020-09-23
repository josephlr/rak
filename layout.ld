PHDRS
{
  ro  PT_LOAD FILEHDR PHDRS ;
  note PT_NOTE ;
  rx  PT_LOAD ;
  rw  PT_LOAD ;
}

ram_min = 1M;

SECTIONS
{
  /* Mapping the program headers and note into RAM makes the file smaller. */
  . = ram_min;
  . += SIZEOF_HEADERS;
  .note : { *(.note) } :note :ro
  .rodata : { *(.rodata .rodata.*) } :ro
  .text   : { *(.text .text.*)     } :rx
  .text32 : { *(.text32)           }
  .data   : { *(.data .data.*)     } :rw
  .bss    : { *(.bss .bss.*)       }
  .stack  : { *(.stack)            }

  /* Comment out for debug information */
  /* /DISCARD/ : {
    *(.symtab)
    *(.strtab)
  } */
  /DISCARD/ : {
    /* Sometimes emitted event for panic = "abort" */
    *(.eh_frame)
    *(.eh_frame_hdr)
    /* Inserted of lld, unnecessary */
    *(.comment)
  }
}
