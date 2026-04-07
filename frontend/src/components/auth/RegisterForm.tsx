/**
 * Registration form component with validation.
 */

import { useState } from "react";
import { useForm } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import { Link } from "react-router-dom";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { useAuth } from "@/hooks";
import { registerSchema, type RegisterFormData } from "@/lib/validations/auth";

interface RegisterFormProps {
  onSuccess?: () => void;
}

export function RegisterForm({ onSuccess }: RegisterFormProps) {
  const { register: registerUser } = useAuth();
  const [error, setError] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(false);

  const {
    register,
    handleSubmit,
    formState: { errors },
  } = useForm<RegisterFormData>({
    resolver: zodResolver(registerSchema),
  });

  const onSubmit = async (data: RegisterFormData) => {
    setError(null);
    setIsLoading(true);

    try {
      await registerUser({
        name: data.name,
        email: data.email,
        password: data.password,
      });
      onSuccess?.();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Error al registrarse");
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div className="rounded-lg bg-white p-8 shadow">
      <form onSubmit={handleSubmit(onSubmit)} className="space-y-6">
        <div>
          <Label htmlFor="name">Nombre</Label>
          <Input
            id="name"
            type="text"
            autoComplete="name"
            {...register("name")}
            className="mt-1"
            aria-invalid={errors.name ? "true" : "false"}
          />
          {errors.name && (
            <p className="mt-1 text-sm text-red-600">{errors.name.message}</p>
          )}
        </div>

        <div>
          <Label htmlFor="email">Correo electrónico</Label>
          <Input
            id="email"
            type="email"
            autoComplete="email"
            {...register("email")}
            className="mt-1"
            aria-invalid={errors.email ? "true" : "false"}
          />
          {errors.email && (
            <p className="mt-1 text-sm text-red-600">{errors.email.message}</p>
          )}
        </div>

        <div>
          <Label htmlFor="password">Contraseña</Label>
          <Input
            id="password"
            type="password"
            autoComplete="new-password"
            {...register("password")}
            className="mt-1"
            aria-invalid={errors.password ? "true" : "false"}
          />
          {errors.password && (
            <p className="mt-1 text-sm text-red-600">
              {errors.password.message}
            </p>
          )}
          <p className="mt-1 text-xs text-gray-500">
            Mínimo 8 caracteres, una mayúscula, una minúscula y un número
          </p>
        </div>

        <div>
          <Label htmlFor="confirmPassword">Confirmar contraseña</Label>
          <Input
            id="confirmPassword"
            type="password"
            autoComplete="new-password"
            {...register("confirmPassword")}
            className="mt-1"
            aria-invalid={errors.confirmPassword ? "true" : "false"}
          />
          {errors.confirmPassword && (
            <p className="mt-1 text-sm text-red-600">
              {errors.confirmPassword.message}
            </p>
          )}
        </div>

        {error && (
          <div className="rounded-md bg-red-50 p-4">
            <p className="text-sm text-red-700">{error}</p>
          </div>
        )}

        <Button type="submit" className="w-full" disabled={isLoading}>
          {isLoading ? "Registrando..." : "Registrarse"}
        </Button>
      </form>

      <p className="mt-6 text-center text-sm text-gray-600">
        ¿Ya tienes cuenta?{" "}
        <Link
          to="/login"
          className="font-medium text-primary hover:text-primary/80"
        >
          Iniciar sesión
        </Link>
      </p>
    </div>
  );
}
