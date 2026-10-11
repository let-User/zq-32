section .multiboot
align 8
    dd 0xE85250D6
    dd 0
    dd header_end - header_start
    dd -(0xE85250D6 + 0 + (header_end - header_start))
header_start:
    dw 0    ; end tag
    dw 0
    dd 8
header_end:

section .bss
align 4096
p4_table:
    resb 4096
p3_table:
    resb 4096
p2_table:
    resb 4096
align 16
stack_bottom:
    resb 16384
stack_top:

section .rodata
align 8
gdt64:
    dq 0
.code equ $ - gdt64
    dq (1<<43) | (1<<44) | (1<<47) | (1<<53)
.pointer:
    dw $ - gdt64 - 1
    dq gdt64

section .text
bits 32
global _start
extern kernel_main

_start:
    cli
    mov esp, stack_top

    ; сохраняем указатель на multiboot info из ebx
    mov edi, ebx

    ; ---- строим таблицы страниц (identity map первых 1 GiB) ----
    mov eax, p3_table
    or  eax, 0b11
    mov [p4_table], eax

    mov eax, p2_table
    or  eax, 0b11
    mov [p3_table], eax

    xor ecx, ecx
.map_p2:
    mov eax, 0x200000
    mul ecx
    or  eax, 0b10000011        ; present | writable | huge
    mov [p2_table + ecx * 8], eax
    inc ecx
    cmp ecx, 512
    jne .map_p2

    ; ---- включаем PAE ----
    mov eax, cr4
    or  eax, 1 << 5
    mov cr4, eax

    ; ---- LME ----
    mov ecx, 0xC0000080
    rdmsr
    or  eax, 1 << 8
    wrmsr

    ; ---- включаем paging ----
    mov eax, p4_table
    or  eax, 0b11
    mov cr3, eax

    mov eax, cr0
    or  eax, 1 << 31
    mov cr0, eax

    ; ---- GDT и прыжок в long mode ----
    lgdt [gdt64.pointer]
    jmp gdt64.code:long_mode_start

bits 64
long_mode_start:
    xor ax, ax
    mov ss, ax
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax

    mov rsp, stack_top

    ; zero-extend edi -> rdi (в 32-бит режиме верхние 32 бита мусорные)
    mov edi, edi

    call kernel_main

    cli
.hang:
    hlt
    jmp .hang
