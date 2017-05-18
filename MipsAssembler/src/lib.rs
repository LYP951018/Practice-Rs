
#[macro_use]
extern crate lazy_static;

pub mod Assembler;

#[cfg(test)]
mod tests {
    use Assembler::Parser;
    #[test]
    fn without_jump_with_normal_offset() {
        let statements = "addiu $sp, $sp, -12
        sw $a0, 8($sp)";
        let mut parser = Parser::new();
        let res = parser.AsmLines(statements.lines());
        assert!(res.is_ok());
        let res = res.unwrap();
        assert!(res.len() == 2);
        assert_eq!(res[0], 0x27bdfff4);
        assert_eq!(res[1], 0xafa40008);
    }

    #[test]
    fn without_jump_without_normal_offset() {
        let statements = "addiu $sp, $sp, -12
        sw $a0, ($sp)";
        let mut parser = Parser::new();
        let res = parser.AsmLines(statements.lines());
        assert!(res.is_ok());
        let res = res.unwrap();
        assert!(res.len() == 2);
        assert_eq!(res[0], 0x27bdfff4);
        assert_eq!(res[1], 0xafa40000);
    }

    #[test]
    fn with_jump() {
        let statements = "  .text
    .globl  main
main:                                    # @sum
# BB#0:                                 # %entry
    addiu $sp, $sp, -12
    sw $a0, 8($sp)
    addiu $v0, $0, 0
    sw $v0, 4($sp)
    addiu $v0, $0, 1
    sw $v0, ($sp)
.LBB0_1:                                # %for.cond
                                        # =>This Inner Loop Header: Depth=1
    lw $v0, ($sp)
    lw $v1, 8($sp)
    slt	$v0, $v0, $v1
    beq $v0, $0, .LBB0_4
    j .LBB0_2
.LBB0_2:                                # %for.body
                                        #   in Loop: Header=BB0_1 Depth=1
    lw $v0, ($sp)
    lw $v1, 4($sp)
    addu $v0, $v1, $v0
    sw $v0, 4($sp)
# BB#3:                                 # %for.inc
                                        #   in Loop: Header=BB0_1 Depth=1
    lw $v0, ($sp)
    addiu $v0, $v0, 1
    sw $v0, ($sp)
    j .LBB0_1
.LBB0_4:                                # %for.end
    lw $v0, 4($sp)
    addiu $sp, $sp, 12";
        let mut parser = Parser::new();
        let res = parser.AsmLines(statements.lines());
        println!("{:?}", res);
        assert!(res.is_ok());
        let res = res.unwrap();
        assert_eq!(res[0], 0x27bdfff4);
        assert_eq!(res[1], 0xafa40008);
        assert_eq!(res[2], 0x24020000);
        assert_eq!(res[3], 0xafa20004);
        assert_eq!(res[9], 0x1040000a);
        assert_eq!(res[res.len() - 3], 0x08000006);
    }
}
