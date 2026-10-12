section .multiboot
align 8
    ; Multiboot2 magic
    dd 0xE85250D6
    ; Architecture: i386 protected mode
    dd 0
    ; Header length
    dd header_end - header_start
    ; Checksum
    dd -(0xE85250D6 + 0 + (header_end - header_start))

header_start:
    ; ---- Framebuffer request tag (type 5) ----
    dw 5            ; type
    dw 0x3          ; flags (optional + set mode)
    dd 20           ; size
    dd 1024         ; width
    dd 768          ; height
    dd 32           ; depth        ; depth

    align 8

    dw 0
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
p2_table2:
    resb 4096
p2_table3:
    resb 4096
p2_table4:
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
   ; --- p4[0] -> p3 ---
    mov eax, p3_table
    or  eax, 0b11
    mov [p4_table], eax

    ; --- p3[0..3] -> p2_table[1..4] ---
    mov eax, p2_table
    or  eax, 0b11
    mov [p3_table], eax

    mov eax, p2_table2
    or  eax, 0b11
    mov [p3_table + 8], eax

    mov eax, p2_table3
    or  eax, 0b11
    mov [p3_table + 16], eax

    mov eax, p2_table4
    or  eax, 0b11
    mov [p3_table + 24], eax

    ; --- заполнить p2_table (гигабайт 0, физические 0..1 GiB) ---
    xor ecx, ecx
.map_p2_1:
    mov eax, 0x200000
    mul ecx
    or  eax, 0b10000011
    mov [p2_table + ecx * 8], eax
    inc ecx
    cmp ecx, 512
    jne .map_p2_1

    ; --- p2_table2 (1..2 GiB) ---
    xor ecx, ecx
.map_p2_2:
    mov eax, 0x200000
    mul ecx
    add eax, 0x40000000        ; смещение 1 GiB
    or  eax, 0b10000011
    mov [p2_table2 + ecx * 8], eax
    inc ecx
    cmp ecx, 512
    jne .map_p2_2

    ; --- p2_table3 (2..3 GiB) ---
    xor ecx, ecx
.map_p2_3:
    mov eax, 0x200000
    mul ecx
    add eax, 0x80000000        ; смещение 2 GiB
    or  eax, 0b10000011
    mov [p2_table3 + ecx * 8], eax
    inc ecx
    cmp ecx, 512
    jne .map_p2_3

    ; --- p2_table4 (3..4 GiB) ---
    xor ecx, ecx
.map_p2_4:
    mov eax, 0x200000
    mul ecx
    add eax, 0xC0000000        ; смещение 3 GiB
    or  eax, 0b10000011
    mov [p2_table4 + ecx * 8], eax
    inc ecx
    cmp ecx, 512
    jne .map_p2_4

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
