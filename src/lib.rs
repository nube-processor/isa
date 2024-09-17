//! TODO

use std::mem::size_of;

use thiserror::Error;

/// Retorna os bits presentes no valor `v` que estão no intervalo `r`.
/// A contagem começa do *low bit* para o *high bit*.
///
/// ## Exemplo:
///
/// ```
/// use isa::*;
///
/// //bits:   543210
/// let v = 0b101000;
/// assert_eq!(0b101, bits(v, 3..=5));
/// ```
pub fn bits<R: std::ops::RangeBounds<usize>>(v: usize, r: R) -> usize {
    let start = match r.start_bound() {
        std::ops::Bound::Included(&n) => n,
        std::ops::Bound::Unbounded => 0,
        std::ops::Bound::Excluded(_) => unreachable!(),
    };

    let end = match r.end_bound() {
        std::ops::Bound::Included(&n) => n,
        std::ops::Bound::Excluded(&n) => n - 1,
        std::ops::Bound::Unbounded => size_of::<usize>() * 8,
    };

    let mask = (1 << (end - start + 1)) - 1;
    let ret = v;
    (ret >> start) & mask
}

/// Altera os bits presentes no valor `v` que estão no intervalo `r` pelos bits `b`.
/// A contagem começa do *low bit* para o *high bit*.
///
/// ## Exemplo:
///
/// ```
/// use isa::*;
///
/// //bits:   543210
/// let v = 0b101000;
/// assert_eq!(0b101110, set_bits(v, 0b11, 1..=2));
/// ```
pub fn set_bits<R: std::ops::RangeBounds<usize>>(v: usize, b: usize, r: R) -> usize {
    let start = match r.start_bound() {
        std::ops::Bound::Included(&n) => n,
        std::ops::Bound::Unbounded => 0,
        std::ops::Bound::Excluded(_) => unreachable!(),
    };

    let end = match r.end_bound() {
        std::ops::Bound::Included(&n) => n,
        std::ops::Bound::Excluded(&n) => n - 1,
        std::ops::Bound::Unbounded => size_of::<usize>() * 8,
    };

    let len = end - start + 1;
    let zeros = !(((1 << len) - 1) << start);

    let val = v & zeros;

    (b << start) | val
}

#[derive(Error, Debug, PartialEq)]
#[error("Instrução inválida: {code}")]
pub struct InvalidInstruction {
    code: usize,
}

macro_rules! instruction_set {
    ($($(#[$doc:meta])* $name:ident $code:literal $mask:literal),+) => {

        /// Conjunto de instruções presentes na Arquitetura do Processador ICMC.
        #[derive(Debug, Copy, Clone, PartialEq)]
        pub enum Instruction {
            $(
                $(#[$doc])*
                #[doc = "# Máscara\n"]
                #[doc = "```txt"]
                #[doc = stringify!($mask)]
                #[doc = "```"]
                $name = $code
            ),+
        }

        impl std::fmt::Display for Instruction {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
                match self {
                    $(Instruction::$name => write!(f, "{}", stringify!($name))),+,
                }
            }
        }

        impl Instruction {

            /// Retorna o OPCODE da instrução.
            pub fn opcode(&self) -> usize {
                let code = match self {
                    $(Instruction::$name => $code),+,
                };
                bits(code, 10..=15)
            }

            /// Retorna a máscara da instrução.
            pub fn mask(&self) -> usize {
                match self {
                    $(Instruction::$name => $code),+,
                }
            }

           /// Retorna qual [`Instruction`] está presente no argumento `v`.
           /// Se a instrução for inválida, irá retornar [`Instruction::InvalidInstruction`].
           ///
           /// ## Exemplo
           ///
           /// ```
           /// use isa::*;
           ///
           /// let mem = 0b1100001000100011; // LOAD
           /// assert_eq!(Instruction::LOAD, Instruction::get_instruction(mem).unwrap());
           /// ```
            pub fn get_instruction(v: usize) -> Result<Instruction, InvalidInstruction> {
                $(if (v & $mask) == $code {
                    return Ok(Instruction::$name);
                })+

                Err(InvalidInstruction { code: v })
            }

        }
    };
}

instruction_set!(
    /// Carrega o valor da memória presente no endereço `END` para o registrador `Rx`.
    ///
    /// # Operação
    /// `Rx` ← MEM(`END`)
    ///
    /// # Uso
    /// ```asm
    /// LOAD Rx, END
    /// ```
    ///
    /// # Exemplo
    /// ```asm
    /// LOAD R3, 0xff00
    /// ```
    LOAD        0b110000_000_000_000_0      0b111111_000_000_000_0, // Data Manipulation Instruction

    /// Carrega o valor `NR` no registrador `Rx`.
    ///
    /// # Operação
    /// `Rx` ← `NR`
    ///
    /// # Uso
    /// ```asm
    /// LOADN Rx, #NR
    /// ```
    ///
    /// # Exemplo
    /// ```asm
    /// LOADN R3, #0xff00
    /// ```
    LOADN       0b111000_000_000_000_0      0b111111_000_000_000_0,

    /// Carrega o valor da memória presente no endereço armazenado em `Ry` para o registrador
    /// `Rx`.
    ///
    /// # Operação
    /// `Rx` ← MEM(`Ry`)
    ///
    /// # Uso
    /// ```asm
    /// LOADI Rx, Ry
    /// ```
    ///
    /// # Exemplo
    /// ```asm
    /// LOADI R3, R0
    /// ```
    LOADI       0b111100_000_000_000_0      0b111111_000_000_000_0,

    /// Salva no endereço `END` da memória o valor presente no registrador `Rx`.
    ///
    /// # Operação
    /// MEM(`END`) ← `Rx`
    ///
    /// # Uso
    /// ```asm
    /// STORE END, Rx
    /// ```
    ///
    /// # Exemplo
    /// ```asm
    /// STORE 0x00ff, R3
    /// ```
    STORE       0b110001_000_000_000_0      0b111111_000_000_000_0,

    /// Salva no endereço `END` da memória o valor `NR`.
    ///
    /// # Operação
    /// MEM(`END`) ← `NR`
    ///
    /// # Uso
    /// ```asm
    /// STOREN END, #NR
    /// ```
    ///
    /// # Exemplo
    /// ```asm
    /// STOREN 0x00ff, #0b10100
    /// ```
    STOREN      0b111001_000_000_000_0      0b111111_000_000_000_0,

    /// Salva, na memória, no endereço armazenado em `Rx`, o valor presente no registrador `Ry`.
    ///
    /// # Operação
    /// MEM(`Rx`) ← `Ry`
    ///
    /// # Uso
    /// ```asm
    /// STOREI Rx, Ry
    /// ```
    ///
    /// # Exemplo
    /// ```asm
    /// STOREI R3, R0
    /// ```
    STOREI      0b111101_000_000_000_0      0b111111_000_000_000_0,

    /// Move, para um registrador `Rx` ou para o `SP`, o valor presente em outro registrador.
    ///
    /// # Operação
    /// `Rx` ← `Ry` ou
    /// `Rx` ← `SP` ou
    /// `SP` ← `Rx`
    ///
    /// # Uso
    /// ```asm
    /// MOV Rx, Ry
    /// MOV Rx, SP
    /// MOV SP, Rx
    /// ```
    ///
    /// # Exemplo
    /// ```asm
    /// MOV R3, R0
    /// MOV R3, SP
    /// MOV SP, R0
    /// ```
    MOV         0b110011_000_000_000_0      0b111111_000_000_000_0,

    INPUT       0b111110_000_000_000_0      0b111111_000_000_000_0, // Peripheric Instructions
    OUTPUT      0b111111_000_000_000_0      0b111111_000_000_000_0,

    /// Imprime na tela do processador um *char* mapeado de um arquivo *charmap*. O código do
    /// *pixelmap* que representa o desenho do *char* está codificado no *low-byte* do registrador
    /// `Rx`, enquanto que sua cor se encontra no *high-byte*. A posição do *char* é armazenada
    /// no registrador `Ry`.
    ///
    /// # Cores
    /// As cores mapeadas atualmente com seus respectivos códigos são:
    /// 1. <span style="background-color:white">⠀⠀</span> White --- 0
    /// 2. <span style="background-color:brown">⠀⠀</span> Brown --- 256
    /// 3. <span style="background-color:green">⠀⠀</span> Green --- 512
    /// 4. <span style="background-color:olive">⠀⠀</span> Olive --- 768
    /// 5. <span style="background-color:navy">⠀⠀</span> Navy --- 1024
    /// 6. <span style="background-color:purple">⠀⠀</span> Purple --- 1280
    /// 7. <span style="background-color:teal">⠀⠀</span> Teal --- 1536
    /// 8. <span style="background-color:silver">⠀⠀</span> Silver --- 1792
    /// 9. <span style="background-color:gray">⠀⠀</span> Gray --- 2048
    /// 10. <span style="background-color:red">⠀⠀</span> Red --- 2304
    /// 11. <span style="background-color:lime">⠀⠀</span> Lime --- 2560
    /// 12. <span style="background-color:yellow">⠀⠀</span> Yellow --- 2816
    /// 13. <span style="background-color:blue">⠀⠀</span> Blue --- 3072
    /// 14. <span style="background-color:fuchsia">⠀⠀</span> Fuchsia --- 3328
    /// 15. <span style="background-color:aqua">⠀⠀</span> Aqua --- 3584
    /// 16. <span style="background-color:black">⠀⠀</span> Black --- 3840
    ///
    /// Para imprimir o caracter colorido, basta somar o código do *char* ao código da cor.
    ///
    /// ## Exemplo
    /// * <span style="color:blue">A</span> --- 37 (código da letra A) + 3072 (código da cor azul).
    ///
    /// # Operação
    /// VÍDEO(`Ry`) ← CHAR(`Rx`)
    ///
    /// # Uso
    /// ```asm
    /// OUTCHAR Rx, Ry
    /// ```
    ///
    /// # Exemplo
    /// ```asm
    /// OUTCHAR R1, R0
    /// ```
    OUTCHAR     0b110010_000_000_000_0      0b111111_000_000_000_0, // IO Instructions

    INCHAR      0b110101_000_000_000_0      0b111111_000_000_000_0,
    SOUND       0b110100_000_000_000_0      0b111111_000_000_000_0,

    /// Realiza a soma dos valores presentes nos registradores `Ry` e `Rz`, guardando o resultado
    /// no registrador `Rx`.
    ///
    /// # Operação
    /// `Rx` ← `Ry` + `Rz`
    ///
    /// # Uso
    /// ```asm
    /// ADD Rx, Ry, Rz
    /// ```
    ///
    /// # Exemplo
    /// ```asm
    /// ADD R3, R0, R7
    /// ```
    ADD         0b100000_000_000_000_0      0b111111_000_000_000_1, // Aritmethic Instructions

    /// Realiza a soma dos valores presentes nos registradores `Ry` e `Rz` mais o *carry* (`C`),
    /// guardando o resultado no registrador `Rx`.
    ///
    /// # Operação
    /// `Rx` ← `Ry` + `Rz` + `C`
    ///
    /// # Uso
    /// ```asm
    /// ADDC Rx, Ry, Rz
    /// ```
    ///
    /// # Exemplo
    /// ```asm
    /// ADDC R3, R0, R7
    /// ```
    ADDC        0b100000_000_000_000_1      0b111111_000_000_000_1,

    /// Realiza a subtração dos valores presentes nos registradores `Ry` e `Rz`, guardando o
    /// resultado no registrador `Rx`.
    ///
    /// # Operação
    /// `Rx` ← `Ry` - `Rz`
    ///
    /// # Uso
    /// ```asm
    /// SUB Rx, Ry, Rz
    /// ```
    ///
    /// # Exemplo
    /// ```asm
    /// SUB R3, R0, R7
    /// ```
    SUB         0b100001_000_000_000_0      0b111111_000_000_000_1,

    /// Realiza a subtração dos valores presentes nos registradores `Ry` e `Rz`, guardando no
    /// registrador `Rx` o resultado somado com o *carry* (`C`).
    ///
    /// # Operação
    /// `Rx` ← `Ry` - `Rz` + `C`
    ///
    /// # Uso
    /// ```asm
    /// SUBC Rx, Ry, Rz
    /// ```
    ///
    /// # Exemplo
    /// ```asm
    /// SUBC R3, R0, R7
    /// ```
    SUBC        0b100001_000_000_000_1      0b111111_000_000_000_1,

    /// Realiza a multiplicação dos valores presentes nos registradores `Ry` e `Rz`, guardando o
    /// resultado no registrador `Rx`.
    ///
    /// # Operação
    /// `Rx` ← `Ry` * `Rz`
    ///
    /// # Uso
    /// ```asm
    /// MUL Rx, Ry, Rz
    /// ```
    ///
    /// # Exemplo
    /// ```asm
    /// MUL R3, R0, R7
    /// ```
    MUL         0b100010_000_000_000_0      0b111111_000_000_000_1,

    /// Realiza a divisão de `Ry` por `Rz`, guardando o resultado no registrador `Rx`.
    ///
    /// # Operação
    /// `Rx` ← `Ry` / `Rz`
    ///
    /// # Uso
    /// ```asm
    /// DIV Rx, Ry, Rz
    /// ```
    ///
    /// # Exemplo
    /// ```asm
    /// DIV R3, R0, R7
    /// ```
    DIV         0b100011_000_000_000_0     0b111111_000_000_000_1,

    /// Incrementa em uma unidade o registrador `Rx`.
    ///
    /// # Operação
    /// `Rx` ← `Rx` + 1
    ///
    /// # Uso
    /// ```asm
    /// INC Rx
    /// ```
    ///
    /// # Exemplo
    /// ```asm
    /// INC R3
    /// ```
    INC         0b100100_000_000_000_0      0b111111_000_100_000_0,

    /// Decrementa em uma unidade o registrador `Rx`.
    ///
    /// # Operação
    /// `Rx` ← `Rx` - 1
    ///
    /// # Uso
    /// ```asm
    /// DEC Rx
    /// ```
    ///
    /// # Exemplo
    /// ```asm
    /// DEC R3
    /// ```
    DEC         0b100100_000_100_000_0      0b111111_000_100_000_0,

    /// Realiza a operação de módulo entre os registradores `Ry` e `Rz` e salva o resultado no
    /// registrador `Rx`.
    ///
    /// # Operação
    /// `Rx` ← `Ry` % `Rz`
    ///
    /// # Uso
    /// ```asm
    /// MOD Rx, Ry, Rz
    /// ```
    ///
    /// # Exemplo
    /// ```asm
    /// MOD R3, R2, R5
    /// ```
    MOD         0b100101_000_000_000_0      0b111111_000_000_000_0,

    /// Realiza a operação *AND* entre os registradores `Ry` e `Rz` e salva o resultado no
    /// registrador `Rx`.
    ///
    /// # Operação
    /// `Rx` ← `Ry` & `Rz`
    ///
    /// # Uso
    /// ```asm
    /// AND Rx, Ry, Rz
    /// ```
    ///
    /// # Exemplo
    /// ```asm
    /// AND R3, R2, R5
    /// ```
    AND         0b010010_000_000_000_0      0b111111_000_000_000_0, // Logic Instructions

    /// Realiza a operação *OR* entre os registradores `Ry` e `Rz` e salva o resultado no
    /// registrador `Rx`.
    ///
    /// # Operação
    /// `Rx` ← `Ry` | `Rz`
    ///
    /// # Uso
    /// ```asm
    /// OR Rx, Ry, Rz
    /// ```
    ///
    /// # Exemplo
    /// ```asm
    /// OR R3, R2, R5
    /// ```
    OR          0b010011_000_000_000_0      0b111111_000_000_000_0,

    /// Realiza a operação *XOR* entre os registradores `Ry` e `Rz` e salva o resultado no
    /// registrador `Rx`.
    ///
    /// # Operação
    /// `Rx` ← `Ry` ^ `Rz`
    ///
    /// # Uso
    /// ```asm
    /// XOR Rx, Ry, Rz
    /// ```
    ///
    /// # Exemplo
    /// ```asm
    /// XOR R3, R2, R5
    /// ```
    XOR         0b010100_000_000_000_0      0b111111_000_000_000_0,

    /// Realiza a operação *NOT* no registrador `Ry` e salva o resultado no registrador `Rx`.
    ///
    /// # Operação
    /// `Rx` ← !`Ry`
    ///
    /// # Uso
    /// ```asm
    /// NOT Rx, Ry
    /// ```
    ///
    /// # Exemplo
    /// ```asm
    /// NOT R3, R2
    /// ```
    NOT         0b010101_000_000_000_0      0b111111_000_000_000_0,

    /// Esta operação desliza os bits para a esquerda `N` vezes e os bits que transbordam a
    /// extremidade esquerda desaparecem. Os espaços na direita são preenchidos com 0.
    ///
    /// # Operação
    /// `Rx` ← `Rx` << `N`
    /// ```txt
    /// 1 0 1 0'0 1 1 1
    ///  ╱ ╱ ╱ ╱ ╱ ╱ ╱
    /// 0 1 0 0'1 1 1 0
    /// ```
    ///
    /// # Uso
    /// ```asm
    /// SHIFTL0 Rx, N
    /// ```
    ///
    /// # Exemplo
    /// ```asm
    /// SHIFTL0 R7, 9
    /// ```
    SHIFTL0     0b010000_000_000_000_0      0b111111_000_111_000_0,

    /// Esta operação desliza os bits para a esquerda `N` vezes e os bits que transbordam a
    /// extremidade esquerda desaparecem. Os espaços na direita são preenchidos com 1.
    ///
    /// # Operação
    /// `Rx` ← !(!(`Rx`) << `N`)
    /// ```txt
    /// 0 0 1 0'0 1 1 1
    ///  ╱ ╱ ╱ ╱ ╱ ╱ ╱
    /// 0 1 0 0'1 1 1 1
    /// ```
    ///
    /// # Uso
    /// ```asm
    /// SHIFTL1 Rx, N
    /// ```
    ///
    /// # Exemplo
    /// ```asm
    /// SHIFTL1 R7, 9
    /// ```
    SHIFTL1     0b010000_000_001_000_0      0b111111_000_111_000_0,

    /// Esta operação desliza os bits para a direita `N` vezes e os bits que transbordam a
    /// extremidade direita desaparecem. Os espaços na esquerda são preenchidos com 0.
    ///
    /// # Operação
    /// `Rx` ← `Rx` >> `N`
    /// ```txt
    /// 0 0 1 0'0 1 1 1
    ///  ╲ ╲ ╲ ╲ ╲ ╲ ╲
    /// 0 0 0 1'0 0 1 1
    /// ```
    ///
    /// # Uso
    /// ```asm
    /// SHIFTR0 Rx, N
    /// ```
    ///
    /// # Exemplo
    /// ```asm
    /// SHIFTR0 R7, 9
    /// ```
    SHIFTR0     0b010000_000_010_000_0      0b111111_000_111_000_0,

    /// Esta operação desliza os bits para a direita `N` vezes e os bits que transbordam a
    /// extremidade direita desaparecem. Os espaços na esquerda são preenchidos com 1.
    ///
    /// # Operação
    /// `Rx` ← !(!(`Rx`) >> `N`)
    /// ```txt
    /// 0 0 1 0'0 1 1 1
    ///  ╲ ╲ ╲ ╲ ╲ ╲ ╲
    /// 0 0 0 1'0 0 1 1
    /// ```
    ///
    /// # Uso
    /// ```asm
    /// SHIFTR1 Rx, N
    /// ```
    ///
    /// # Exemplo
    /// ```asm
    /// SHIFTR1 R7, 9
    /// ```
    SHIFTR1     0b010000_000_011_000_0      0b111111_000_111_000_0,

    /// Esta operação gira os bits para a esquerda `N` vezes e os bits que transbordam para
    /// a extremidade esquerda são reintroduzidos no lado direito.
    ///
    /// # Operação
    /// `Rx` ← (`Rx` << N) | (`Rx` >> ([`BITS_ADDRESS`] - N))
    /// ```txt
    /// ╭─────────────────╮
    /// │ 0 0 1 0'0 1 1 1 │
    /// ╰─╯╱ ╱ ╱ ╱ ╱ ╱ ╱╭─╯
    ///   0 1 0 0'1 1 1 0
    /// ```
    ///
    /// # Uso
    /// ```asm
    /// ROTL Rx, N
    /// ```
    ///
    /// # Exemplo
    /// ```asm
    /// ROTL R6, 2
    /// ```
    ROTL        0b010000_000_100_000_0      0b111111_000_110_000_0,

    /// Esta operação gira os bits para a direita `N` vezes e os bits que transbordam para
    /// a extremidade direita são reintroduzidos no lado esquerdo.
    ///
    /// # Operação
    /// `Rx` ← (`Rx` >> N) | (`Rx` << ([`BITS_ADDRESS`] - N))
    /// ```txt
    /// ╭─────────────────╮
    /// │ 0 0 1 0'0 1 1 1 │
    /// ╰─╮╲ ╲ ╲ ╲ ╲ ╲ ╲╰─╯
    ///   1 0 0 1'0 0 1 1
    /// ```
    ///
    /// # Uso
    /// ```asm
    /// ROTL Rx, N
    /// ```
    ///
    /// # Exemplo
    /// ```asm
    /// ROTL R6, 2
    /// ```
    ROTR        0b010000_000_110_000_0      0b111111_000_110_000_0,

    /// Compara os valores dos registradores `Rx` e `Ry` e atualiza o *flag register* (`FR`) de
    /// acordo com o resultado.
    ///
    /// # Operação
    /// `FR` ← `COND`
    ///
    /// # Uso
    /// ```asm
    /// CMP Rx, Ry
    /// ```
    ///
    /// # Exemplo
    /// ```asm
    /// CMP R3, R2
    /// ```
    CMP         0b010110_000_000_000_0      0b111111_000_000_000_0,

    /// Pula para o endereço `END` da memória.
    ///
    /// # Operação
    /// `PC` ← `END`
    ///
    /// # Uso
    /// ```asm
    /// JMP END
    /// ```
    ///
    /// # Exemplo
    /// ```asm
    /// JMP 0x00ff
    /// ```
    JMP         0b000010_000_000_000_0      0b111111_111_100_000_0,

    /// Pula para o endereço `END` da memória **se** o *bit* [`FlagIndex::EQUAL`] do
    /// *flag register* estiver setado.
    ///
    /// # Operação
    /// `PC` ← `END`
    ///
    /// # Uso
    /// ```asm
    /// JEQ END
    /// ```
    ///
    /// # Exemplo
    /// ```asm
    /// JEQ 0x00ff
    /// ```
    JEQ         0b000010_000_100_000_0      0b111111_111_100_000_0,

    /// Pula para o endereço `END` da memória **se** o *bit* [`FlagIndex::EQUAL`] do
    /// *flag register* não estiver setado.
    ///
    /// # Operação
    /// `PC` ← `END`
    ///
    /// # Uso
    /// ```asm
    /// JNE END
    /// ```
    ///
    /// # Exemplo
    /// ```asm
    /// JNE 0x00ff
    /// ```
    JNE         0b000010_001_000_000_0      0b111111_111_100_000_0,

    /// Pula para o endereço `END` da memória **se** o *bit* [`FlagIndex::ZERO`] do
    /// *flag register* estiver setado.
    ///
    /// # Operação
    /// `PC` ← `END`
    ///
    /// # Uso
    /// ```asm
    /// JZ END
    /// ```
    ///
    /// # Exemplo
    /// ```asm
    /// JZ 0x00ff
    /// ```
    JZ          0b000010_001_100_000_0      0b111111_111_100_000_0,

    /// Pula para o endereço `END` da memória **se** o *bit* [`FlagIndex::ZERO`] do
    /// *flag register* não estiver setado.
    ///
    /// # Operação
    /// `PC` ← `END`
    ///
    /// # Uso
    /// ```asm
    /// JNZ END
    /// ```
    ///
    /// # Exemplo
    /// ```asm
    /// JNZ 0x00ff
    /// ```
    JNZ         0b000010_010_000_000_0      0b111111_111_100_000_0,

    /// Pula para o endereço `END` da memória **se** o *bit* [`FlagIndex::CARRY`] do
    /// *flag register* estiver setado.
    ///
    /// # Operação
    /// `PC` ← `END`
    ///
    /// # Uso
    /// ```asm
    /// JC END
    /// ```
    ///
    /// # Exemplo
    /// ```asm
    /// JC 0x00ff
    /// ```
    JC          0b000010_010_100_000_0      0b111111_111_100_000_0,

    /// Pula para o endereço `END` da memória **se** o *bit* [`FlagIndex::CARRY`] do
    /// *flag register* não estiver setado.
    ///
    /// # Operação
    /// `PC` ← `END`
    ///
    /// # Uso
    /// ```asm
    /// JNC END
    /// ```
    ///
    /// # Exemplo
    /// ```asm
    /// JNC 0x00ff
    /// ```
    JNC         0b000010_011_000_000_0      0b111111_111_100_000_0,

    /// Pula para o endereço `END` da memória **se** o *bit* [`FlagIndex::GREATER`] do
    /// *flag register* estiver setado.
    ///
    /// # Operação
    /// `PC` ← `END`
    ///
    /// # Uso
    /// ```asm
    /// JGR END
    /// ```
    ///
    /// # Exemplo
    /// ```asm
    /// JGR 0x00ff
    /// ```
    JGR         0b000010_011_100_000_0      0b111111_111_100_000_0,

    /// Pula para o endereço `END` da memória **se** o *bit* [`FlagIndex::LESSER`] do
    /// *flag register* estiver setado.
    ///
    /// # Operação
    /// `PC` ← `END`
    ///
    /// # Uso
    /// ```asm
    /// JLE END
    /// ```
    ///
    /// # Exemplo
    /// ```asm
    /// JLE 0x00ff
    /// ```
    JLE         0b000010_100_000_000_0      0b111111_111_100_000_0,

    /// Pula para o endereço `END` da memória **se** algum dos *bits* [`FlagIndex::GREATER`] ou
    /// [`FlagIndex::EQUAL`] do *flag register* estiver setado.
    ///
    /// # Operação
    /// `PC` ← `END`
    ///
    /// # Uso
    /// ```asm
    /// JEG END
    /// ```
    ///
    /// # Exemplo
    /// ```asm
    /// JEG 0x00ff
    /// ```
    JEG         0b000010_100_100_000_0      0b111111_111_100_000_0,

    /// Pula para o endereço `END` da memória **se** algum dos *bits* [`FlagIndex::LESSER`] ou
    /// [`FlagIndex::EQUAL`] do *flag register* estiver setado.
    ///
    /// # Operação
    /// `PC` ← `END`
    ///
    /// # Uso
    /// ```asm
    /// JEL END
    /// ```
    ///
    /// # Exemplo
    /// ```asm
    /// JEL 0x00ff
    /// ```
    JEL         0b000010_101_000_000_0      0b111111_111_100_000_0,

    /// Pula para o endereço `END` da memória **se** o *bit* [`FlagIndex::ARITHMETIC_OVERFLOW`] do
    /// *flag register* estiver setado.
    ///
    /// # Operação
    /// `PC` ← `END`
    ///
    /// # Uso
    /// ```asm
    /// JOV END
    /// ```
    ///
    /// # Exemplo
    /// ```asm
    /// JOV 0x00ff
    /// ```
    JOV         0b000010_101_100_000_0      0b111111_111_100_000_0,

    /// Pula para o endereço `END` da memória **se** o *bit* [`FlagIndex::ARITHMETIC_OVERFLOW`] do
    /// *flag register* não estiver setado.
    ///
    /// # Operação
    /// `PC` ← `END`
    ///
    /// # Uso
    /// ```asm
    /// JNO END
    /// ```
    ///
    /// # Exemplo
    /// ```asm
    /// JNO 0x00ff
    /// ```
    JNO         0b000010_110_000_000_0      0b111111_111_100_000_0,

    /// Pula para o endereço `END` da memória **se** o *bit* [`FlagIndex::DIV_BY_ZERO`] do
    /// *flag register* estiver setado.
    ///
    /// # Operação
    /// `PC` ← `END`
    ///
    /// # Uso
    /// ```asm
    /// JDZ END
    /// ```
    ///
    /// # Exemplo
    /// ```asm
    /// JDZ 0x00ff
    /// ```
    JDZ         0b000010_110_100_000_0      0b111111_111_100_000_0,

    /// Pula para o endereço `END` da memória **se** o *bit* [`FlagIndex::NEGATIVE`] do
    /// *flag register* estiver setado.
    ///
    /// # Operação
    /// `PC` ← `END`
    ///
    /// # Uso
    /// ```asm
    /// JN END
    /// ```
    ///
    /// # Exemplo
    /// ```asm
    /// JN 0x00ff
    /// ```
    JN          0b000010_111_000_000_0      0b111111_111_100_000_0,

    /// Salva o valor atual do *PC* na *stack* e pula para o endereço do procedimento informado.
    ///
    /// # Operação
    /// MEM(`SP`) ← `PC`
    /// `PC` ← `END`
    /// `SP` ← `SP` - 1
    ///
    /// # Uso
    /// ```asm
    /// CALL END
    /// ```
    ///
    /// # Exemplo
    /// ```asm
    /// CALL 0x003C
    /// ```
    CALL        0b000011_000_000_000_0     0b111111_111_100_000_0,

    /// Salva o valor atual do *PC* na *stack* e pula para o endereço do procedimento informado se
    /// o *bit* [`FlagIndex::EQUAL`] do *flag register* estiver setado.
    ///
    /// # Operação
    /// MEM(`SP`) ← `PC`
    /// `PC` ← `END`
    /// `SP` ← `SP` - 1
    ///
    /// # Uso
    /// ```asm
    /// CEQ END
    /// ```
    ///
    /// # Exemplo
    /// ```asm
    /// CEQ 0x003C
    /// ```
    CEQ         0b000011_000_100_000_0      0b111111_111_100_000_0,

    /// Salva o valor atual do *PC* na *stack* e pula para o endereço do procedimento informado se
    /// o *bit* [`FlagIndex::EQUAL`] do *flag register* não estiver setado.
    ///
    /// # Operação
    /// MEM(`SP`) ← `PC`
    /// `PC` ← `END`
    /// `SP` ← `SP` - 1
    ///
    /// # Uso
    /// ```asm
    /// CNE END
    /// ```
    ///
    /// # Exemplo
    /// ```asm
    /// CNE 0x003C
    /// ```
    CNE         0b000011_001_000_000_0      0b111111_111_100_000_0,

    /// Salva o valor atual do *PC* na *stack* e pula para o endereço do procedimento informado se
    /// o *bit* [`FlagIndex::ZERO`] do *flag register* estiver setado.
    ///
    /// # Operação
    /// MEM(`SP`) ← `PC`
    /// `PC` ← `END`
    /// `SP` ← `SP` - 1
    ///
    /// # Uso
    /// ```asm
    /// CZ END
    /// ```
    ///
    /// # Exemplo
    /// ```asm
    /// CZ 0x003C
    /// ```
    CZ          0b000011_001_100_000_0      0b111111_111_100_000_0,

    /// Salva o valor atual do *PC* na *stack* e pula para o endereço do procedimento informado se
    /// o *bit* [`FlagIndex::ZERO`] do *flag register* não estiver setado.
    ///
    /// # Operação
    /// MEM(`SP`) ← `PC`
    /// `PC` ← `END`
    /// `SP` ← `SP` - 1
    ///
    /// # Uso
    /// ```asm
    /// CNZ END
    /// ```
    ///
    /// # Exemplo
    /// ```asm
    /// CNZ 0x003C
    /// ```
    CNZ         0b000011_010_000_000_0      0b111111_111_100_000_0,

    /// Salva o valor atual do *PC* na *stack* e pula para o endereço do procedimento informado se
    /// o *bit* [`FlagIndex::CARRY`] do *flag register* estiver setado.
    ///
    /// # Operação
    /// MEM(`SP`) ← `PC`
    /// `PC` ← `END`
    /// `SP` ← `SP` - 1
    ///
    /// # Uso
    /// ```asm
    /// CC END
    /// ```
    ///
    /// # Exemplo
    /// ```asm
    /// CC 0x003C
    /// ```
    CC          0b000011_010_100_000_0      0b111111_111_100_000_0,

    /// Salva o valor atual do *PC* na *stack* e pula para o endereço do procedimento informado se
    /// o *bit* [`FlagIndex::CARRY`] do *flag register* não estiver setado.
    ///
    /// # Operação
    /// MEM(`SP`) ← `PC`
    /// `PC` ← `END`
    /// `SP` ← `SP` - 1
    ///
    /// # Uso
    /// ```asm
    /// CNC END
    /// ```
    ///
    /// # Exemplo
    /// ```asm
    /// CNC 0x003C
    /// ```
    CNC         0b000011_011_000_000_0      0b111111_111_100_000_0,

    /// Salva o valor atual do *PC* na *stack* e pula para o endereço do procedimento informado se
    /// o *bit* [`FlagIndex::GREATER`] do *flag register* estiver setado.
    ///
    /// # Operação
    /// MEM(`SP`) ← `PC`
    /// `PC` ← `END`
    /// `SP` ← `SP` - 1
    ///
    /// # Uso
    /// ```asm
    /// CGR END
    /// ```
    ///
    /// # Exemplo
    /// ```asm
    /// CGR 0x003C
    /// ```
    CGR         0b000011_011_100_000_0      0b111111_111_100_000_0,

    /// Salva o valor atual do *PC* na *stack* e pula para o endereço do procedimento informado se
    /// o *bit* [`FlagIndex::LESSER`] do *flag register* estiver setado.
    ///
    /// # Operação
    /// MEM(`SP`) ← `PC`
    /// `PC` ← `END`
    /// `SP` ← `SP` - 1
    ///
    /// # Uso
    /// ```asm
    /// CLE END
    /// ```
    ///
    /// # Exemplo
    /// ```asm
    /// CLE 0x003C
    /// ```
    CLE         0b000011_100_000_000_0      0b111111_111_100_000_0,

    /// Salva o valor atual do *PC* na *stack* e pula para o endereço do procedimento informado se
    /// algum dos *bits* [`FlagIndex::EQUAL`] ou [`FlagIndex::GREATER`] do *flag register* estiver
    /// setado
    ///
    /// # Operação
    /// MEM(`SP`) ← `PC`
    /// `PC` ← `END`
    /// `SP` ← `SP` - 1
    ///
    /// # Uso
    /// ```asm
    /// CEG END
    /// ```
    ///
    /// # Exemplo
    /// ```asm
    /// CEG 0x003C
    /// ```
    CEG         0b000011_100_100_000_0      0b111111_111_100_000_0,

    /// Salva o valor atual do *PC* na *stack* e pula para o endereço do procedimento informado se
    /// algum dos *bits* [`FlagIndex::EQUAL`] ou [`FlagIndex::LESSER`] do *flag register* estiver
    /// setado
    ///
    /// # Operação
    /// MEM(`SP`) ← `PC`
    /// `PC` ← `END`
    /// `SP` ← `SP` - 1
    ///
    /// # Uso
    /// ```asm
    /// CEL END
    /// ```
    ///
    /// # Exemplo
    /// ```asm
    /// CEL 0x003C
    /// ```
    CEL         0b000011_101_000_000_0      0b111111_111_100_000_0,

    /// Salva o valor atual do *PC* na *stack* e pula para o endereço do procedimento informado se
    /// o *bit* [`FlagIndex::ARITHMETIC_OVERFLOW`] do *flag register* estiver setado.
    ///
    /// # Operação
    /// MEM(`SP`) ← `PC`
    /// `PC` ← `END`
    /// `SP` ← `SP` - 1
    ///
    /// # Uso
    /// ```asm
    /// COV END
    /// ```
    ///
    /// # Exemplo
    /// ```asm
    /// COV 0x003C
    /// ```
    COV         0b000011_101_100_000_0      0b111111_111_100_000_0,

    /// Salva o valor atual do *PC* na *stack* e pula para o endereço do procedimento informado se
    /// o *bit* [`FlagIndex::ARITHMETIC_OVERFLOW`] do *flag register* não estiver setado.
    ///
    /// # Operação
    /// MEM(`SP`) ← `PC`
    /// `PC` ← `END`
    /// `SP` ← `SP` - 1
    ///
    /// # Uso
    /// ```asm
    /// CNO END
    /// ```
    ///
    /// # Exemplo
    /// ```asm
    /// CNO 0x003C
    /// ```
    CNO         0b000011_110_000_000_0      0b111111_111_100_000_0,

    /// Salva o valor atual do *PC* na *stack* e pula para o endereço do procedimento informado se
    /// o *bit* [`FlagIndex::DIV_BY_ZERO`] do *flag register* estiver setado.
    ///
    /// # Operação
    /// MEM(`SP`) ← `PC`
    /// `PC` ← `END`
    /// `SP` ← `SP` - 1
    ///
    /// # Uso
    /// ```asm
    /// CDZ END
    /// ```
    ///
    /// # Exemplo
    /// ```asm
    /// CDZ 0x003C
    /// ```
    CDZ         0b000011_110_100_000_0      0b111111_111_100_000_0,

    /// Salva o valor atual do *PC* na *stack* e pula para o endereço do procedimento informado se
    /// o *bit* [`FlagIndex::NEGATIVE`] do *flag register* estiver setado.
    ///
    /// # Operação
    /// MEM(`SP`) ← `PC`
    /// `PC` ← `END`
    /// `SP` ← `SP` - 1
    ///
    /// # Uso
    /// ```asm
    /// CN END
    /// ```
    ///
    /// # Exemplo
    /// ```asm
    /// CN 0x003C
    /// ```
    CN          0b000011_111_000_000_0      0b111111_111_100_000_0,

    /// Altera o valor do *PC* para o último valor salvo na *stack* somado de 1.
    ///
    /// # Operação
    /// `SP` ← `SP` + 1
    /// `PC` ← MEM(`SP`)
    /// `PC` ← `PC` + 1
    ///
    /// # Uso
    /// ```asm
    /// RTS
    /// ```
    ///
    /// # Exemplo
    /// ```asm
    /// RTS
    /// ```
    RTS         0b000100_000_000_000_0      0b111111_000_000_000_1,

    /// Altera o valor do *PC* para o último valor salvo na *stack*.
    ///
    /// # Operação
    /// `SP` ← `SP` + 1
    /// `PC` ← MEM(`SP`)
    ///
    /// # Uso
    /// ```asm
    /// RTI
    /// ```
    ///
    /// # Exemplo
    /// ```asm
    /// RTI
    /// ```
    RTI         0b000100_000_000_000_1      0b111111_000_000_000_1,

    /// Salva na *stack* o conteúdo de um registrador ou do *flag register*.
    ///
    /// # Operação
    /// MEM(`SP`) ← `Rx`
    /// `SP` ← `SP` - 1 ou
    /// MEM(`SP`) ← `FR`
    /// `SP` ← `SP` - 1
    ///
    /// # Uso
    /// ```asm
    /// PUSH Rx
    /// PUSH FR
    /// ```
    ///
    /// # Exemplo
    /// ```asm
    /// PUSH R5
    /// PUSH FR
    /// ```
    PUSH        0b000101_000_000_000_0      0b111111_000_000_000_0,

    /// Recupera da *stack* o conteúdo de um registrador ou do *flag register*.
    ///
    /// # Operação
    /// `SP` ← `SP` + 1
    /// `Rx` ← MEM(`SP`) ou
    /// `SP` ← `SP` + 1
    /// `FR` ← MEM(`SP`)
    ///
    /// # Uso
    /// ```asm
    /// POP Rx
    /// POP FR
    /// ```
    ///
    /// # Exemplo
    /// ```asm
    /// POP R5
    /// POP FR
    /// ```
    POP         0b000110_000_000_000_0      0b111111_000_000_000_0,

    /// Sem operação. Serve apenas para consumir tempo.
    ///
    /// # Operação
    /// Nenhuma
    ///
    /// # Uso
    /// ```asm
    /// NOP
    /// ```
    ///
    /// # Exemplo
    /// ```asm
    /// NOP
    /// ```
    NOP         0b000000_000_000_000_0      0b111111_000_000_000_0, // Control Instructions

    /// Para a execução do programa.
    ///
    /// # Operação
    /// Para o processador
    ///
    /// # Uso
    /// ```asm
    /// HALT
    /// ```
    ///
    /// # Exemplo
    /// ```asm
    /// HALT
    /// ```
    HALT        0b001111_000_000_000_0      0b111111_000_000_000_0,

    /// Limpa o bit [`FlagIndex::CARRY`] do *flag register*.
    ///
    /// # Operação
    /// C ← 0
    ///
    /// # Uso
    /// ```asm
    /// CLEARC
    /// ```
    ///
    /// # Exemplo
    /// ```asm
    /// CLEARC
    /// ```
    CLEARC      0b001000_000_000_000_0      0b111111_100_000_000_0,

    /// Seta o bit [`FlagIndex::CARRY`] do *flag register*.
    ///
    /// # Operação
    /// C ← 1
    ///
    /// # Uso
    /// ```asm
    /// SETC
    /// ```
    ///
    /// # Exemplo
    /// ```asm
    /// SETC
    /// ```
    SETC        0b001000_100_000_000_0      0b111111_100_000_000_0,

    /// Gera um *breakpoint* no código, forçando o simulador a entrar no modo *debug*.
    ///
    /// # Operação
    /// Nenhuma operação lógica no processador
    ///
    /// # Uso
    /// ```asm
    /// BREAKP
    /// ```
    ///
    /// # Exemplo
    /// ```asm
    /// BREAKP
    /// ```
    BREAKP      0b001110_000_000_000_0      0b111111_000_000_000_0
);

impl Default for Instruction {
    fn default() -> Self {
        Self::NOP
    }
}

#[cfg(test)]
mod tests {
    use crate::set_bits;

    use super::*;

    #[test]
    fn test_set_bits() {
        let value = 0b_110_001_000_usize;
        let bits = 0b010_usize;
        assert_eq!(set_bits(value, bits, 3..=5), 0b_110_010_000_usize)
    }

    #[test]
    fn test_opcode() {
        let inst = Instruction::LOAD;
        assert_eq!(inst.opcode(), 0b110000);
    }

    #[test]
    fn test_get_instruction() {
        let code = 0b100000_111_000_000_1; // ADDC
        assert_eq!(
            Instruction::get_instruction(code).unwrap(),
            Instruction::ADDC
        );
    }
}
