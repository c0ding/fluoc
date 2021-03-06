use std::fmt::Display;
use std::path;
use std::process;

extern crate inkwell;
use inkwell::passes::PassManager;
use inkwell::targets::{InitializationConfig, Target};
use inkwell::AddressSpace;
use inkwell::{builder, context, module};

fn error<T>(msg: String, err: T) -> !
where
    T: Display,
{
    eprintln!("\x1b[31m{}\x1b[0m {}", msg, err);
    process::exit(1);
}

pub struct Generator<'a> {
    pub module: module::Module<'a>,
    pub context: &'a context::Context,
    pub builder: builder::Builder<'a>,
}

impl<'a> Generator<'a> {
    fn new<'b>(context: &'b context::Context, module: module::Module<'b>) -> Generator<'b> {
        let fpm = PassManager::create(&module);

        fpm.add_instruction_combining_pass();
        fpm.add_reassociate_pass();
        fpm.add_gvn_pass();
        fpm.add_cfg_simplification_pass();
        fpm.add_basic_alias_analysis_pass();
        fpm.add_promote_memory_to_register_pass();
        fpm.add_instruction_combining_pass();
        fpm.add_reassociate_pass();
        fpm.add_tail_call_elimination_pass();

        fpm.initialize();

        Generator {
            context,
            module,
            builder: context.create_builder(),
        }
    }

    fn generate_fmt(&mut self) {
        self.generate_printf();
        self.generate_print_int();
        self.generate_print_long();
    }

    fn generate_printf(&mut self) {
        // C printf - NOT for external use, only for use by the builtins
        let i32_type = self.context.i32_type();
        let i8_type = self.context.i8_type();

        let first_param_type = i8_type.ptr_type(AddressSpace::Generic);
        let fn_type = i32_type.fn_type(&[first_param_type.into()], true);

        self.module.add_function("printf", fn_type, None);
    }

    fn generate_print_int(&mut self) {
        // print_int ( int ) -> ()
        let i32_type = self.context.i32_type();
        let empty_tuple = self.context.struct_type(&[], false);

        let fn_type = empty_tuple.fn_type(&[i32_type.into()], false);
        let fn_addr = self.module.add_function("print_int", fn_type, None);

        let entry_block = self.context.append_basic_block(fn_addr, "entry");

        self.builder.position_at_end(entry_block);

        let format = self.builder.build_global_string_ptr("%i\n", "format");

        self.builder.build_call(
            self.module
                .get_function("printf")
                .expect("There is no printf defined??"),
            &[
                inkwell::values::BasicValueEnum::PointerValue(format.as_pointer_value()),
                fn_addr.get_nth_param(0).unwrap().into_int_value().into(),
            ],
            "temp2",
        );

        self.builder
            .build_return(Some(&inkwell::values::BasicValueEnum::StructValue(
                empty_tuple.const_named_struct(&[]),
            )));
    }

    fn generate_print_long(&mut self) {
        // print_int ( long ) -> ()
        let i64_type = self.context.i64_type();
        let empty_tuple = self.context.struct_type(&[], false);

        let fn_type = empty_tuple.fn_type(&[i64_type.into()], false);
        let fn_addr = self.module.add_function("print_long", fn_type, None);

        let entry_block = self.context.append_basic_block(fn_addr, "entry");

        self.builder.position_at_end(entry_block);

        let format = self.builder.build_global_string_ptr("%ld\n", "format");

        self.builder.build_call(
            self.module
                .get_function("printf")
                .expect("There is no printf defined??"),
            &[
                inkwell::values::BasicValueEnum::PointerValue(format.as_pointer_value()),
                fn_addr.get_nth_param(0).unwrap().into_int_value().into(),
            ],
            "temp2",
        );

        self.builder
            .build_return(Some(&inkwell::values::BasicValueEnum::StructValue(
                empty_tuple.const_named_struct(&[]),
            )));
    }

    fn generate_op(&mut self) {
        self.generate_add_int();
        self.generate_mul_int();
        self.generate_sub_int();
        self.generate_cmp_int();

        self.generate_add_long();
        self.generate_mul_long();
        self.generate_sub_long();
        self.generate_cmp_long();
    }

    fn generate_add_int(&mut self) {
        let i32_type = self.context.i32_type();

        let fn_type = i32_type.fn_type(&[i32_type.into(), i32_type.into()], false);
        let fn_addr = self.module.add_function("add_int", fn_type, None);

        let entry_block = self.context.append_basic_block(fn_addr, "entry");

        self.builder.position_at_end(entry_block);

        let add_result = self.builder.build_int_add(
            fn_addr.get_nth_param(0).unwrap().into_int_value().into(),
            fn_addr.get_nth_param(1).unwrap().into_int_value().into(),
            "int_addition_temp",
        );

        self.builder
            .build_return(Some(&inkwell::values::BasicValueEnum::IntValue(add_result)));
    }

    fn generate_mul_int(&mut self) {
        let i32_type = self.context.i32_type();

        let fn_type = i32_type.fn_type(&[i32_type.into(), i32_type.into()], false);
        let fn_addr = self.module.add_function("mul_int", fn_type, None);

        let entry_block = self.context.append_basic_block(fn_addr, "entry");

        self.builder.position_at_end(entry_block);

        let add_result = self.builder.build_int_mul(
            fn_addr.get_nth_param(0).unwrap().into_int_value().into(),
            fn_addr.get_nth_param(1).unwrap().into_int_value().into(),
            "int_multiplication_temp",
        );

        self.builder
            .build_return(Some(&inkwell::values::BasicValueEnum::IntValue(add_result)));
    }

    fn generate_sub_int(&mut self) {
        let i32_type = self.context.i32_type();

        let fn_type = i32_type.fn_type(&[i32_type.into(), i32_type.into()], false);
        let fn_addr = self.module.add_function("sub_int", fn_type, None);

        let entry_block = self.context.append_basic_block(fn_addr, "entry");

        self.builder.position_at_end(entry_block);

        let add_result = self.builder.build_int_sub(
            fn_addr.get_nth_param(0).unwrap().into_int_value().into(),
            fn_addr.get_nth_param(1).unwrap().into_int_value().into(),
            "int_subtraction_temp",
        );

        self.builder
            .build_return(Some(&inkwell::values::BasicValueEnum::IntValue(add_result)));
    }

    fn generate_cmp_int(&mut self) {
        let key_vals = [
            ("equal_to_int", inkwell::IntPredicate::EQ),
            ("less_than_int", inkwell::IntPredicate::SLT),
            ("greater_than_int", inkwell::IntPredicate::SGT),
            ("greater_than_eq_int", inkwell::IntPredicate::SGE),
            ("less_than_eq_int", inkwell::IntPredicate::SLE),
        ];

        for (func_name, pred) in key_vals.iter() {
            let i32_type = self.context.i32_type();
            let bool_type = self.context.bool_type();

            let fn_type = bool_type.fn_type(&[i32_type.into(), i32_type.into()], false);
            let fn_addr = self.module.add_function(func_name, fn_type, None);

            let entry_block = self.context.append_basic_block(fn_addr, "entry");

            self.builder.position_at_end(entry_block);

            let cmp_result = self.builder.build_int_compare(
                *pred,
                fn_addr.get_nth_param(0).unwrap().into_int_value().into(),
                fn_addr.get_nth_param(1).unwrap().into_int_value().into(),
                &format!("{}_temp", func_name)[..],
            );

            self.builder
                .build_return(Some(&inkwell::values::BasicValueEnum::IntValue(cmp_result)));
        }
    }

    fn generate_add_long(&mut self) {
        let i64_type = self.context.i64_type();

        let fn_type = i64_type.fn_type(&[i64_type.into(), i64_type.into()], false);
        let fn_addr = self.module.add_function("add_long", fn_type, None);

        let entry_block = self.context.append_basic_block(fn_addr, "entry");

        self.builder.position_at_end(entry_block);

        let add_result = self.builder.build_int_add(
            fn_addr.get_nth_param(0).unwrap().into_int_value().into(),
            fn_addr.get_nth_param(1).unwrap().into_int_value().into(),
            "int_addition_temp",
        );

        self.builder
            .build_return(Some(&inkwell::values::BasicValueEnum::IntValue(add_result)));
    }

    fn generate_mul_long(&mut self) {
        let i64_type = self.context.i64_type();

        let fn_type = i64_type.fn_type(&[i64_type.into(), i64_type.into()], false);
        let fn_addr = self.module.add_function("mul_long", fn_type, None);

        let entry_block = self.context.append_basic_block(fn_addr, "entry");

        self.builder.position_at_end(entry_block);

        let add_result = self.builder.build_int_mul(
            fn_addr.get_nth_param(0).unwrap().into_int_value().into(),
            fn_addr.get_nth_param(1).unwrap().into_int_value().into(),
            "int_multiplication_temp",
        );

        self.builder
            .build_return(Some(&inkwell::values::BasicValueEnum::IntValue(add_result)));
    }

    fn generate_sub_long(&mut self) {
        let i64_type = self.context.i64_type();

        let fn_type = i64_type.fn_type(&[i64_type.into(), i64_type.into()], false);
        let fn_addr = self.module.add_function("sub_long", fn_type, None);

        let entry_block = self.context.append_basic_block(fn_addr, "entry");

        self.builder.position_at_end(entry_block);

        let add_result = self.builder.build_int_sub(
            fn_addr.get_nth_param(0).unwrap().into_int_value().into(),
            fn_addr.get_nth_param(1).unwrap().into_int_value().into(),
            "int_subtraction_temp",
        );

        self.builder
            .build_return(Some(&inkwell::values::BasicValueEnum::IntValue(add_result)));
    }

    fn generate_cmp_long(&mut self) {
        let key_vals = [
            ("equal_to_long", inkwell::IntPredicate::EQ),
            ("less_than_long", inkwell::IntPredicate::SLT),
            ("greater_than_long", inkwell::IntPredicate::SGT),
            ("greater_than_eq_long", inkwell::IntPredicate::SGE),
            ("less_than_eq_long", inkwell::IntPredicate::SLE),
        ];

        for (func_name, pred) in key_vals.iter() {
            let i64_type = self.context.i64_type();
            let bool_type = self.context.bool_type();

            let fn_type = bool_type.fn_type(&[i64_type.into(), i64_type.into()], false);
            let fn_addr = self.module.add_function(func_name, fn_type, None);

            let entry_block = self.context.append_basic_block(fn_addr, "entry");

            self.builder.position_at_end(entry_block);

            let cmp_result = self.builder.build_int_compare(
                *pred,
                fn_addr.get_nth_param(0).unwrap().into_int_value().into(),
                fn_addr.get_nth_param(1).unwrap().into_int_value().into(),
                &format!("{}_temp", func_name)[..],
            );

            self.builder
                .build_return(Some(&inkwell::values::BasicValueEnum::IntValue(cmp_result)));
        }
    }
}

macro_rules! generate_llvm {
    ($core_path: expr, $context: expr, $func: expr, $name: expr) => {
        let mut gen = Generator::new(&$context, $context.create_module($name));
        $func(&mut gen);
        let path = $core_path.join(format!("{}.bc", $name));
        gen.module.write_bitcode_to_path(&path);

        match gen
            .module
            .print_to_file($core_path.join(format!("{}.ll", $name)))
        {
            Ok(_) => {}
            Err(e) => error("LLVM build error:".to_string(), e.to_string()),
        };
    };
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    let mut core_path = path::PathBuf::from(file!());

    core_path.pop();
    core_path.push("src");
    core_path.push("fluo_core");
    core_path.push("core");

    let llvm_context = context::Context::create();
    Target::initialize_native(&InitializationConfig::default())
        .expect("Failed to initialize native target");

    generate_llvm!(core_path, llvm_context, Generator::generate_fmt, "fmt");
    // generate_llvm!(core_path, llvm_context, Generator::generate_op, "op");
}
