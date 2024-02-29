use super::StaticKernelSource;
use crate::{
    codegen::{execute_static, EagerHandle, WorkgroupLaunch},
    element::JitElement,
    tensor::JitTensor,
    Runtime,
};

/// Creates a unary kernel.
#[macro_export]
macro_rules! unary {
    (
        operation: $ops:expr,
        runtime: $runtime:ty,
        input: $input:expr,
        elem: $elem:ty
    ) => {{
        unary!(operation: $ops, compiler: <$runtime as Runtime>::Compiler);

        $crate::kernel::unary::<
            Ops<<$runtime as Runtime>::Compiler, $elem>,
            OpsInplace<<$runtime as Runtime>::Compiler, $elem>,
            $runtime,
            $elem,
            D
        >($input, None, true)
    }};
    (
        operation: $ops:expr,
        runtime: $runtime:ty,
        input: $input:expr; $scalar:expr,
        elem: $elem:ty
    ) => {{
        unary!(operation: $ops, compiler: <$runtime as Runtime>::Compiler, scalar 1);

        $crate::kernel::unary::<
            Ops<<$runtime as Runtime>::Compiler, $elem>,
            OpsInplace<<$runtime as Runtime>::Compiler, $elem>,
            $runtime,
            $elem,
            D
        >($input, Some(&[$scalar]), true)
    }};

    (
        operation: $ops:expr,
        compiler: $compiler:ty
    ) => {
        pub struct Ops<C, E> {
            _c: core::marker::PhantomData<C>,
            _e: core::marker::PhantomData<E>,
        }
        pub struct OpsInplace<C, E> {
            _c: core::marker::PhantomData<C>,
            _e: core::marker::PhantomData<E>,
        }

        #[allow(clippy::redundant_closure_call)]
        fn compile<C, E>(
            settings: $crate::codegen::CompilationSettings,
        ) -> $crate::kernel::SourceTemplate
        where
            C: $crate::codegen::Compiler,
            E: $crate::element::JitElement
        {

            let mut scope = $crate::codegen::dialect::gpu::Scope::root();
            let op = $ops(&mut scope, E::gpu_elem());
            scope.register(op);

            let local = scope.last_local_index().unwrap().index().unwrap();

            let input = $crate::codegen::InputInfo::Array {
                item: $crate::codegen::dialect::gpu::Item::Scalar(E::gpu_elem()),
                visibility: $crate::codegen::dialect::gpu::Visibility::Read,
            };
            let out = $crate::codegen::OutputInfo::ArrayWrite {
                item: $crate::codegen::dialect::gpu::Item::Scalar(E::gpu_elem()),
                local,
            };
            let info = $crate::codegen::CompilationInfo {
                inputs: vec![input],
                outputs: vec![out],
                scope,
            };
            let shader = $crate::codegen::Compilation::new(info).compile(settings);

            let compiled = C::compile(shader);
            $crate::kernel::SourceTemplate::new(compiled.to_string())
        }

        #[allow(clippy::redundant_closure_call)]
        impl<C, E> $crate::kernel::StaticKernelSource for Ops<C, E>
        where
            C: $crate::codegen::Compiler,
            E: $crate::element::JitElement,
        {
            fn source() -> $crate::kernel::SourceTemplate {
                let settings = $crate::codegen::CompilationSettings::default();
                compile::<C, E>(settings)
            }
        }

        #[allow(clippy::redundant_closure_call)]
        impl<C, E> $crate::kernel::StaticKernelSource for OpsInplace<C, E>
        where
            C: $crate::codegen::Compiler,
            E: $crate::element::JitElement,
        {
            fn source() -> $crate::kernel::SourceTemplate {
                let mapping = $crate::codegen::InplaceMapping {
                    pos_input: 0,
                    pos_output: 0,
                };
                let settings = $crate::codegen::CompilationSettings::default()
                    .inplace(vec![mapping]);
                compile::<C, E>(settings)
            }
        }
    };
    (
        operation: $ops:expr,
        compiler: $compiler:ty,
        scalar $num:expr
    ) => {
        pub struct Ops<C, E> {
            _c: core::marker::PhantomData<C>,
            _e: core::marker::PhantomData<E>,
        }
        pub struct OpsInplace<C, E> {
            _c: core::marker::PhantomData<C>,
            _e: core::marker::PhantomData<E>,
        }

        #[allow(clippy::redundant_closure_call)]
        fn compile<C, E>(
            settings: $crate::codegen::CompilationSettings,
        ) -> $crate::kernel::SourceTemplate
        where
            C: $crate::codegen::Compiler,
            E: $crate::element::JitElement
        {

            let mut scope = $crate::codegen::dialect::gpu::Scope::root();
            let op = $ops(&mut scope, E::gpu_elem());
            scope.register(op);

            let local = scope.last_local_index().unwrap().index().unwrap();

            let input = $crate::codegen::InputInfo::Array {
                item: $crate::codegen::dialect::gpu::Item::Scalar(E::gpu_elem()),
                visibility: $crate::codegen::dialect::gpu::Visibility::Read,
            };
            let scalars = $crate::codegen::InputInfo::Scalar {
                elem: E::gpu_elem(),
                size: $num,
            };
            let out = $crate::codegen::OutputInfo::ArrayWrite {
                item: $crate::codegen::dialect::gpu::Item::Scalar(E::gpu_elem()),
                local,
            };
            let info = $crate::codegen::CompilationInfo {
                inputs: vec![input, scalars],
                outputs: vec![out],
                scope,
            };
            let shader = $crate::codegen::Compilation::new(info).compile(settings);

            let compiled = C::compile(shader);
            $crate::kernel::SourceTemplate::new(compiled.to_string())
        }

        #[allow(clippy::redundant_closure_call)]
        impl<C, E> $crate::kernel::StaticKernelSource for Ops<C, E>
        where
            C: $crate::codegen::Compiler,
            E: $crate::element::JitElement,
        {
            fn source() -> $crate::kernel::SourceTemplate {
                let settings = $crate::codegen::CompilationSettings::default();
                compile::<C, E>(settings)
            }
        }

        #[allow(clippy::redundant_closure_call)]
        impl<C, E> $crate::kernel::StaticKernelSource for OpsInplace<C, E>
        where
            C: $crate::codegen::Compiler,
            E: $crate::element::JitElement,
        {
            fn source() -> $crate::kernel::SourceTemplate {
                let mapping = $crate::codegen::InplaceMapping {
                    pos_input: 0,
                    pos_output: 0,
                };
                let settings = $crate::codegen::CompilationSettings::default()
                    .inplace(vec![mapping]);
                compile::<C, E>(settings)
            }
        }
    };
}

/// Launch an unary operation.
pub fn unary<Kernel, KernelInplace, R: Runtime, E, const D: usize>(
    tensor: JitTensor<R, E, D>,
    scalars: Option<&[E]>,
    inplace_enabled: bool,
) -> JitTensor<R, E, D>
where
    Kernel: StaticKernelSource,
    KernelInplace: StaticKernelSource,
    E: JitElement,
{
    if inplace_enabled && tensor.can_mut() {
        execute_static::<R, KernelInplace, E>(
            &[EagerHandle::new(
                &tensor.handle,
                &tensor.strides,
                &tensor.shape.dims,
            )],
            &[],
            scalars,
            WorkgroupLaunch::Input { pos: 0 },
            tensor.client.clone(),
        );

        tensor
    } else {
        let num_elems = tensor.shape.num_elements();
        let buffer = tensor.client.empty(num_elems * core::mem::size_of::<E>());
        let output = JitTensor::new(
            tensor.client.clone(),
            tensor.device,
            tensor.shape.clone(),
            buffer,
        );

        execute_static::<R, Kernel, E>(
            &[EagerHandle::new(
                &tensor.handle,
                &tensor.strides,
                &tensor.shape.dims,
            )],
            &[EagerHandle::new(
                &output.handle,
                &output.strides,
                &output.shape.dims,
            )],
            scalars,
            WorkgroupLaunch::Output { pos: 0 },
            tensor.client,
        );

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codegen::dialect::gpu::{Operator, Scope, UnaryOperator};
    use crate::tests::{ReferenceBackend, TestBackend, TestCompiler, TestRuntime};
    use burn_tensor::{Distribution, Tensor};

    unary!(
        operation: |scope: &mut Scope, elem| Operator::Tanh(UnaryOperator {
            input: scope.read_array(0, elem),
            out: scope.create_local(elem),
        }),
        compiler: TestCompiler
    );

    #[test]
    fn unary_should_work_with_multiple_invocations() {
        let tensor =
            Tensor::<TestBackend, 2>::random([6, 256], Distribution::Default, &Default::default());
        let tensor_ref =
            Tensor::<ReferenceBackend, 2>::from_data(tensor.to_data(), &Default::default());

        let actual =
            unary::<Ops<TestCompiler, f32>, OpsInplace<TestCompiler, f32>, TestRuntime, f32, 2>(
                tensor.into_primitive(),
                None,
                true,
            );
        let expected = tensor_ref.tanh();

        expected.into_data().assert_approx_eq(
            &Tensor::<TestBackend, 2>::from_primitive(actual).into_data(),
            3,
        );
    }

    #[test]
    fn unary_inplace_should_work_with_multiple_invocations() {
        let tensor =
            Tensor::<TestBackend, 2>::random([6, 256], Distribution::Default, &Default::default());
        let tensor_ref =
            Tensor::<ReferenceBackend, 2>::from_data(tensor.to_data(), &Default::default());

        let actual =
            unary::<Ops<TestCompiler, f32>, OpsInplace<TestCompiler, f32>, TestRuntime, f32, 2>(
                tensor.into_primitive(),
                None,
                true,
            );
        let expected = tensor_ref.tanh();

        expected.into_data().assert_approx_eq(
            &Tensor::<TestBackend, 2>::from_primitive(actual).into_data(),
            3,
        );
    }

    #[test]
    fn tanh_should_not_have_numerical_bugs_on_macos() {
        fn tanh_one_value(input: f32) -> f32 {
            let tensor = Tensor::<TestBackend, 1>::ones([1], &Default::default()) * input;
            let output = tensor.tanh().into_primitive();
            Tensor::<TestBackend, 1>::from_primitive(output)
                .into_data()
                .value[0]
        }

        let ok = tanh_one_value(43.0); // metal tanh gives 1.0 which is the right answer
        let zero = tanh_one_value(44.0); // metal tanh gives zero when within 43.67..44.36
        let nan = tanh_one_value(45.0); // metal tanh gives nan when over 44.36
        let neg = tanh_one_value(-45.0); //  metal works correctly here

        assert!(!ok.is_nan() && ok == 1.0);
        assert!(!zero.is_nan() && zero == 1.0);
        assert!(!nan.is_nan() && nan == 1.0);
        assert!(!neg.is_nan() && neg == -1.0);
    }
}