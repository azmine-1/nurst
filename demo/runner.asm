; ---------------------------------------------------------------------------
; NURST RUNNER - a demo game for the nurst emulator
;
; NROM (mapper 0), 32 KB PRG, 8 KB CHR, vertical mirroring.
;
; The screen is split with a sprite 0 hit: the top four rows hold a fixed
; status bar while everything below scrolls horizontally across both
; nametables. Obstacles are 16x16 metasprites; sound comes from all four of
; the APU's tone channels.
; ---------------------------------------------------------------------------

; ------------------------------------------------------------- PPU/APU ports
PPUCTRL   = $2000
PPUMASK   = $2001
PPUSTATUS = $2002
OAMADDR   = $2003
PPUSCROLL = $2005
PPUADDR   = $2006
PPUDATA   = $2007
OAMDMA    = $4014
APUSTATUS = $4015
JOYPAD1   = $4016

; ---------------------------------------------------------------- zero page
frame       = $00
nmi_flag    = $01
state       = $02      ; 0 = title, 1 = playing, 2 = game over
state_init  = $03      ; non-zero asks the main loop to redraw the screen
cam_sub     = $04      ; camera X, 8.8 fixed point
cam_lo      = $05
cam_hi      = $06
speed_sub   = $07      ; scroll speed, 8.8 fixed point
speed_int   = $08
py_lo       = $09      ; player Y, 8.8 fixed point
py_hi       = $0A
vy_lo       = $0B      ; player vertical velocity, signed 8.8
vy_hi       = $0C
on_ground   = $0D
anim        = $0E
pad         = $0F
pad_prev    = $10
pad_new     = $11      ; buttons pressed this frame only
spawn_timer = $12
oam_ptr     = $13
tmp0        = $14
tmp1        = $15
tmp2        = $16
tmp3        = $17
rng_lo      = $18
rng_hi      = $19
score_tick  = $1A
music_step  = $1B
music_timer = $1C
ramp_timer  = $1D
obj_index   = $1E      ; kept out of tmp0-tmp3, which push_metasprite uses
score       = $20      ; six digits, most significant first
hiscore     = $26
obj_x       = $30      ; MAX_OBJ entries each
obj_y       = $38
obj_type    = $40      ; 0 = free, 1 = cactus, 2 = bird
obj_sub     = $48

OAM_BUFFER  = $0200

; ----------------------------------------------------------------- constants
MAX_OBJ     = 6
PLAYER_X    = 48
GROUND_Y    = 160      ; player's top edge when standing
BIRD_Y      = 132
GRAVITY     = $40      ; 0.25 px per frame per frame
JUMP_LO     = $80      ; -4.50 px per frame
JUMP_HI     = $FB
MAX_FALL    = 6
START_SPEED = $80      ; 1.5 px per frame
TOP_SPEED   = 4

BTN_A       = $80
BTN_B       = $40
BTN_SELECT  = $20
BTN_START   = $10
BTN_UP      = $08

TILE_SKY    = $01

; Sprite pattern numbers, from tools/tiles.py.
SPR_ZERO    = $01
SPR_RUN_A   = $10
SPR_RUN_B   = $14
SPR_JUMP    = $18
SPR_CACTUS  = $20
SPR_BIRD_A  = $30
SPR_BIRD_B  = $34

    .org $8000

; ---------------------------------------------------------------------- boot
reset:
    sei
    cld
    ldx #$40
    stx $4017              ; silence the APU frame IRQ
    ldx #$FF
    txs
    inx
    stx PPUCTRL            ; disable NMI and rendering while we set up
    stx PPUMASK
    stx $4010              ; and the DMC IRQ

    jsr wait_vblank

    ; Clear the whole of RAM, then park every sprite off screen.
    lda #0
    tax
@clear_ram:
    sta $0000,x
    sta $0100,x
    sta $0300,x
    sta $0400,x
    sta $0500,x
    sta $0600,x
    sta $0700,x
    inx
    bne @clear_ram
    lda #$FF
    ldx #0
@clear_oam:
    sta OAM_BUFFER,x
    inx
    bne @clear_oam

    jsr wait_vblank

    jsr load_palettes
    jsr load_nametables

    lda #$5A               ; any non-zero seed will do
    sta rng_lo
    lda #$C3
    sta rng_hi

    lda #1
    sta state_init         ; ask the main loop to draw the title screen
    lda #0
    sta state

    jsr reset_run

    lda #%00001111         ; enable all four tone channels
    sta APUSTATUS

    lda #$88               ; NMI on, sprites from the second pattern table
    sta PPUCTRL
    lda #%00011110         ; show background and sprites, including column 0
    sta PPUMASK

; ----------------------------------------------------------------- main loop
main:
    jsr wait_nmi

    lda state_init
    beq @playfield
    jsr redraw_screen      ; costs a frame, so skip the split this time
    jmp main

@playfield:
    jsr split_scroll
    jsr read_pad
    jsr update
    jsr build_oam
    jmp main

; --------------------------------------------------------------------- video

wait_vblank:
    bit PPUSTATUS
@wait:
    bit PPUSTATUS
    bpl @wait
    rts

wait_nmi:
    lda #0
    sta nmi_flag
@wait:
    lda nmi_flag
    beq @wait
    rts

; The status bar occupies the top four rows at scroll zero. Sprite 0 sits on
; its bottom edge, so its hit marks the scanline where the camera takes over.
; The polling loops are bounded so a frame with rendering off cannot wedge the
; game. One pass of the inner loop covers about 2800 cycles, and the hit lands
; roughly 3600 cycles after the flag clears, so the counts leave real headroom.
split_scroll:
    ldx #4
@wait_clear:
    ldy #0
@clear_loop:
    lda PPUSTATUS
    and #$40
    beq @armed
    dey
    bne @clear_loop
    dex
    bne @wait_clear
    rts                    ; the hit never cleared: give up on this frame
@armed:
    ldx #8
@wait_hit:
    ldy #0
@hit_loop:
    lda PPUSTATUS
    and #$40
    bne @split
    dey
    bne @hit_loop
    dex
    bne @wait_hit
    rts
@split:
    lda cam_hi
    and #$01               ; bit 8 of the camera picks the nametable
    ora #$88
    sta PPUCTRL
    lda cam_lo
    sta PPUSCROLL
    lda #0
    sta PPUSCROLL
    rts

load_palettes:
    bit PPUSTATUS
    lda #$3F
    sta PPUADDR
    lda #$00
    sta PPUADDR
    ldx #0
@loop:
    lda palette_data,x
    sta PPUDATA
    inx
    cpx #32
    bne @loop
    rts

; Copy both nametables, which together make one seamless 512-pixel loop.
load_nametables:
    bit PPUSTATUS
    lda #$20
    sta PPUADDR
    lda #$00
    sta PPUADDR

    lda #<nametable_data
    sta tmp0
    lda #>nametable_data
    sta tmp1
    ldx #8                 ; 8 pages of 256 bytes = two full nametables
    ldy #0
@loop:
    lda (tmp0),y
    sta PPUDATA
    iny
    bne @loop
    inc tmp1
    dex
    bne @loop
    rts

; Screen text lives in the sky rows of the first nametable, so it has to be
; drawn with rendering off and cleared again before the camera scrolls past.
redraw_screen:
    lda #0
    sta PPUMASK

    jsr clear_text_rows

    lda state
    beq @title
    cmp #1
    beq @done
    ; game over
    lda #<text_gameover
    ldx #>text_gameover
    ldy #11
    jsr draw_centered
    lda #<text_start
    ldx #>text_start
    ldy #15
    jsr draw_centered
    jmp @done
@title:
    lda #<text_title
    ldx #>text_title
    ldy #11
    jsr draw_centered
    lda #<text_start
    ldx #>text_start
    ldy #15
    jsr draw_centered

@done:
    lda #0
    sta state_init
    ; Point the address latch somewhere harmless before rendering resumes.
    bit PPUSTATUS
    lda #$20
    sta PPUADDR
    lda #$00
    sta PPUADDR
    lda #0
    sta PPUSCROLL
    sta PPUSCROLL
    lda #%00011110
    sta PPUMASK
    rts

clear_text_rows:
    bit PPUSTATUS
    lda #$21               ; $2140 is row 10, column 0
    sta PPUADDR
    lda #$40
    sta PPUADDR
    ldx #0
    lda #TILE_SKY
@loop:
    sta PPUDATA
    inx
    cpx #224               ; seven rows
    bne @loop
    rts

; A = string low byte, X = string high byte, Y = nametable row.
draw_centered:
    sta tmp2
    stx tmp3
    sty tmp0

    ldy #0
@measure:
    lda (tmp2),y
    beq @have_length
    iny
    bne @measure
@have_length:
    tya
    lsr a
    sta tmp1               ; half the length, in tiles
    lda #16
    sec
    sbc tmp1               ; starting column, centred on 32
    sta tmp1

    ; VRAM address = $2000 + row * 32 + column
    lda tmp0
    lsr a
    lsr a
    lsr a                  ; row / 8 gives the high byte's low bits
    clc
    adc #$20
    pha
    lda tmp0
    asl a
    asl a
    asl a
    asl a
    asl a                  ; (row * 32) & $FF
    clc
    adc tmp1
    tax
    pla

    bit PPUSTATUS
    sta PPUADDR
    stx PPUADDR
    ldy #0
@loop:
    lda (tmp2),y
    beq @done
    sta PPUDATA
    iny
    bne @loop
@done:
    rts

; -------------------------------------------------------------------- input
read_pad:
    lda pad
    sta pad_prev
    lda #1
    sta JOYPAD1
    lda #0
    sta JOYPAD1
    ldx #8
@loop:
    lda JOYPAD1
    lsr a                  ; button state into carry
    rol pad                ; and into the accumulating byte
    dex
    bne @loop
    lda pad
    eor pad_prev
    and pad
    sta pad_new
    rts

; ---------------------------------------------------------------- game logic

reset_run:
    lda #0
    sta cam_sub
    sta cam_lo
    sta cam_hi
    sta vy_lo
    sta vy_hi
    sta py_lo
    sta anim
    sta score_tick
    sta music_step
    sta music_timer
    sta ramp_timer
    lda #GROUND_Y
    sta py_hi
    lda #1
    sta on_ground
    lda #START_SPEED
    sta speed_sub
    lda #1
    sta speed_int
    lda #60
    sta spawn_timer

    ldx #MAX_OBJ - 1
@clear_objects:
    lda #0
    sta obj_type,x
    dex
    bpl @clear_objects

    ldx #5
@clear_score:
    lda #0
    sta score,x
    dex
    bpl @clear_score
    rts

update:
    inc anim
    lda state
    beq title_update
    cmp #1
    beq play_update
    jmp over_update

title_update:
    lda pad_new
    and #BTN_START
    beq @done
    jsr reset_run
    lda #1
    sta state
    sta state_init
@done:
    rts

over_update:
    lda pad_new
    and #BTN_START
    beq @done
    lda #0
    sta state
    lda #1
    sta state_init
@done:
    rts

play_update:
    jsr move_camera
    jsr move_player
    jsr move_objects
    jsr spawn_objects
    jsr check_collisions
    jsr add_distance_score
    jsr ramp_speed
    jsr play_music
    rts

move_camera:
    lda cam_sub
    clc
    adc speed_sub
    sta cam_sub
    lda cam_lo
    adc speed_int
    sta cam_lo
    lda cam_hi
    adc #0
    sta cam_hi
    rts

move_player:
    lda on_ground
    beq @airborne

    ; On the ground: A or Up starts a jump.
    lda pad_new
    and #BTN_A | BTN_UP
    beq @stay
    lda #JUMP_LO
    sta vy_lo
    lda #JUMP_HI
    sta vy_hi
    lda #0
    sta on_ground
    jsr sfx_jump
    rts
@stay:
    lda #GROUND_Y
    sta py_hi
    lda #0
    sta py_lo
    rts

@airborne:
    lda vy_lo
    clc
    adc #GRAVITY
    sta vy_lo
    lda vy_hi
    adc #0
    sta vy_hi
    bmi @apply             ; still rising, so no terminal velocity check
    cmp #MAX_FALL
    bcc @apply
    lda #MAX_FALL
    sta vy_hi
    lda #0
    sta vy_lo

@apply:
    lda py_lo
    clc
    adc vy_lo
    sta py_lo
    lda py_hi
    adc vy_hi
    sta py_hi

    ; Landing: only while falling, and only once we reach the ground line.
    lda vy_hi
    bmi @ceiling
    lda py_hi
    cmp #GROUND_Y
    bcc @done
    lda #GROUND_Y
    sta py_hi
    lda #0
    sta py_lo
    sta vy_lo
    sta vy_hi
    lda #1
    sta on_ground
    rts

@ceiling:
    lda py_hi
    cmp #40
    bcs @done
    lda #40                ; stop just below the status bar
    sta py_hi
    lda #0
    sta py_lo
    sta vy_lo
    sta vy_hi
@done:
    rts

move_objects:
    ldx #MAX_OBJ - 1
@loop:
    lda obj_type,x
    beq @next
    lda obj_sub,x
    sec
    sbc speed_sub
    sta obj_sub,x
    lda obj_x,x
    sbc speed_int
    sta obj_x,x
    bcs @next
    lda #0                 ; walked off the left edge
    sta obj_type,x
@next:
    dex
    bpl @loop
    rts

spawn_objects:
    dec spawn_timer
    bne @done

    ldx #MAX_OBJ - 1
@find_slot:
    lda obj_type,x
    beq @found
    dex
    bpl @find_slot
    lda #30                ; every slot is busy: try again shortly
    sta spawn_timer
    rts

@found:
    lda #250
    sta obj_x,x
    lda #0
    sta obj_sub,x

    jsr rng
    and #$03
    beq @bird              ; one spawn in four is a bird
    lda #1
    sta obj_type,x
    lda #GROUND_Y
    sta obj_y,x
    jmp @reload
@bird:
    lda #2
    sta obj_type,x
    lda #BIRD_Y
    sta obj_y,x

@reload:
    ; The gap shrinks as the run speeds up, but never below a jump's length.
    jsr rng
    and #$1F
    clc
    adc #45
    sta spawn_timer
@done:
    rts

check_collisions:
    ldx #MAX_OBJ - 1
@loop:
    lda obj_type,x
    beq @next

    lda obj_x,x
    sec
    sbc #PLAYER_X
    clc
    adc #10
    cmp #21                ; horizontal boxes overlap?
    bcs @next

    lda obj_y,x
    sec
    sbc py_hi
    clc
    adc #11
    cmp #23                ; vertical boxes overlap?
    bcs @next

    jmp game_over
@next:
    dex
    bpl @loop
    rts

game_over:
    jsr sfx_crash
    jsr silence_music
    lda #2
    sta state
    lda #1
    sta state_init

    ; Keep the best run so far.
    ldx #0
@compare:
    lda score,x
    cmp hiscore,x
    beq @same_digit
    bcs @new_record
    rts
@same_digit:
    inx
    cpx #6
    bne @compare
    rts
@new_record:
    ldx #5
@copy:
    lda score,x
    sta hiscore,x
    dex
    bpl @copy
    rts

add_distance_score:
    inc score_tick
    lda score_tick
    and #$03
    bne @done
    jsr add_one_point
@done:
    rts

; Add one to the six-digit decimal score, carrying leftwards.
add_one_point:
    ldx #5
@loop:
    inc score,x
    lda score,x
    cmp #10
    bcc @done
    lda #0
    sta score,x
    dex
    bpl @loop
@done:
    rts

ramp_speed:
    inc ramp_timer
    lda ramp_timer
    and #$0F
    bne @done
    lda speed_int
    cmp #TOP_SPEED
    bcs @done
    lda speed_sub
    clc
    adc #4
    sta speed_sub
    lda speed_int
    adc #0
    sta speed_int
@done:
    rts

; ------------------------------------------------------------------- sprites

; Build the display list in RAM; the NMI hands it to the PPU by DMA.
build_oam:
    lda #0
    sta oam_ptr

    ; Sprite 0 marks the scanline where the status bar ends. It is drawn
    ; behind the background, so the divider line hides it.
    ldx oam_ptr
    lda #23
    sta OAM_BUFFER,x
    lda #SPR_ZERO
    sta OAM_BUFFER+1,x
    lda #$23               ; palette 3, behind the background
    sta OAM_BUFFER+2,x
    lda #0
    sta OAM_BUFFER+3,x
    lda #4
    sta oam_ptr

    jsr draw_player

    ldx #0
@objects:
    stx obj_index
    lda obj_type,x
    beq @next
    lda obj_x,x
    sta tmp0
    lda obj_y,x
    sta tmp1

    lda obj_type,x
    cmp #1
    beq @cactus
    ; bird, with two flapping frames
    lda anim
    and #$08
    beq @bird_a
    lda #SPR_BIRD_B
    jmp @bird_go
@bird_a:
    lda #SPR_BIRD_A
@bird_go:
    sta tmp2
    lda #$02               ; sprite palette 2
    jmp @emit
@cactus:
    lda #SPR_CACTUS
    sta tmp2
    lda #$01               ; sprite palette 1
@emit:
    jsr push_metasprite
@next:
    ldx obj_index
    inx
    cpx #MAX_OBJ
    bne @objects

    ; Hide whatever is left of the display list.
    ldx oam_ptr
    lda #$FF
@hide:
    sta OAM_BUFFER,x
    inx
    inx
    inx
    inx
    bne @hide
    rts

draw_player:
    lda #PLAYER_X
    sta tmp0
    lda py_hi
    sta tmp1

    lda on_ground
    beq @jumping
    lda anim
    and #$08               ; alternate the run frames every eight frames
    beq @run_a
    lda #SPR_RUN_B
    jmp @go
@run_a:
    lda #SPR_RUN_A
    jmp @go
@jumping:
    lda #SPR_JUMP
@go:
    sta tmp2
    lda #$00               ; sprite palette 0
    jsr push_metasprite
    rts

; Append a 16x16 metasprite: tmp0 = X, tmp1 = Y, tmp2 = first tile, A = attrs.
push_metasprite:
    sta tmp3
    ldx oam_ptr
    ldy #0
@loop:
    ; Sprites appear one scanline below their OAM Y, so bias it up by one.
    lda tmp1
    sec
    sbc #1
    clc
    adc metasprite_dy,y
    sta OAM_BUFFER,x
    lda tmp2
    clc
    adc metasprite_tile,y
    sta OAM_BUFFER+1,x
    lda tmp3
    sta OAM_BUFFER+2,x
    lda tmp0
    clc
    adc metasprite_dx,y
    sta OAM_BUFFER+3,x
    inx
    inx
    inx
    inx
    iny
    cpy #4
    bne @loop
    stx oam_ptr
    rts

; Tile order matches tools/tiles.py: top-left, bottom-left, top-right,
; bottom-right.
metasprite_dx:
    .byte 0, 0, 8, 8
metasprite_dy:
    .byte 0, 8, 0, 8
metasprite_tile:
    .byte 0, 1, 2, 3

; --------------------------------------------------------------------- audio

sfx_jump:
    lda #$9A               ; 25% duty, constant volume 10
    sta $4000
    lda #$8B               ; sweep up: the pitch rises as the note plays
    sta $4001
    lda #$80
    sta $4002
    lda #$49               ; short length, timer high bit set
    sta $4003
    rts

sfx_crash:
    lda #$3C               ; noise at constant volume 12
    sta $400C
    lda #$08
    sta $400E
    lda #$60
    sta $400F
    rts

silence_music:
    lda #$30               ; volume 0 silences the melody
    sta $4004
    lda #$80               ; clear the triangle's linear counter
    sta $4008
    rts

; One music step every eight frames, walking both patterns in lockstep.
play_music:
    dec music_timer
    bpl @done
    lda #7
    sta music_timer

    ldx music_step
    lda music_bass,x
    beq @no_bass
    tay
    lda #$FF               ; hold the triangle's linear counter open
    sta $4008
    lda note_lo,y
    sta $400A
    lda note_hi,y
    ora #$18               ; reload the length counter
    sta $400B
@no_bass:

    ldx music_step
    lda music_lead,x
    beq @no_lead
    tay
    lda #$5A               ; 25% duty, constant volume 10
    sta $4004
    lda #$08               ; sweep off
    sta $4005
    lda note_lo,y
    sta $4006
    lda note_hi,y
    ora #$18
    sta $4007
@no_lead:

    inc music_step
    lda music_step
    cmp #MUSIC_LENGTH
    bcc @done
    lda #0
    sta music_step
@done:
    rts

; ---------------------------------------------------------------------- misc

; 16-bit Galois LFSR. Returns a pseudo-random byte in A.
rng:
    lsr rng_hi
    ror rng_lo
    bcc @done
    lda rng_hi
    eor #$B4
    sta rng_hi
@done:
    lda rng_lo
    eor rng_hi
    rts

; ----------------------------------------------------------------------- NMI

nmi:
    pha
    txa
    pha
    tya
    pha

    lda #0
    sta OAMADDR
    lda #>OAM_BUFFER
    sta OAMDMA

    jsr draw_status

    ; Park the camera at the top-left for the status bar; the sprite 0 split
    ; hands the rest of the screen back to the scrolling playfield.
    bit PPUSTATUS
    lda #$88
    sta PPUCTRL
    lda #0
    sta PPUSCROLL
    sta PPUSCROLL

    inc frame
    lda #1
    sta nmi_flag

    pla
    tay
    pla
    tax
    pla
    rti

draw_status:
    bit PPUSTATUS
    lda #$20
    sta PPUADDR
    lda #$28               ; row 1, column 8
    sta PPUADDR
    ldx #0
@score:
    lda score,x
    clc
    adc #'0'
    sta PPUDATA
    inx
    cpx #6
    bne @score

    bit PPUSTATUS
    lda #$20
    sta PPUADDR
    lda #$35               ; row 1, column 21
    sta PPUADDR
    ldx #0
@hiscore:
    lda hiscore,x
    clc
    adc #'0'
    sta PPUDATA
    inx
    cpx #6
    bne @hiscore
    rts

irq:
    rti

; ---------------------------------------------------------------------- text
text_title:
    .byte "  NURST RUNNER  ", 0
text_start:
    .byte "  PRESS START  ", 0
text_gameover:
    .byte "  GAME OVER  ", 0

palette_data:
    ; Background: sky, ground, hills, status bar.
    .byte $0F,$21,$30,$3C
    .byte $0F,$07,$17,$2A
    .byte $0F,$21,$09,$1A
    .byte $0F,$0F,$10,$30
    ; Sprites: runner, cactus, bird, and the invisible sprite 0.
    .byte $0F,$0F,$01,$2C
    .byte $0F,$0F,$09,$2A
    .byte $0F,$0F,$06,$27
    .byte $0F,$30,$30,$30
