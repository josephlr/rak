PHDRS
{
  ram  PT_LOAD FILEHDR PHDRS ;
  note PT_NOTE               ;
}

ram_min = 1M;

SECTIONS
{
  /* Mapping the program headers and note into RAM makes the file smaller. */
  . = ram_min;
  . += SIZEOF_HEADERS;
  .note : { *(.note) } :note :ram

  .rodata : { *(.rodata .rodata.*) } :ram
  .text   : { *(.text .text.*)     }
  .text32 : { *(.text32)           }
  .data   : { *(.data .data.*)     }
  .bss    : { *(.bss .bss.*)       }

  /* Comment out for debug information */
  /DISCARD/ : {
    *(.symtab)
    *(.strtab)
  }
  /DISCARD/ : {
    /* Sometimes emitted event for panic = "abort" */
    *(.eh_frame)
    *(.eh_frame_hdr)
    /* Inserted of lld, unnecessary */
    *(.comment)
  }
}
