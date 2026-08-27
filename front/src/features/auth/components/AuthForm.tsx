import Button from "@/components/ui/Button/Button";
import React, { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { useAuth } from "@/features/auth/hooks/useAuth";
import PasswordField from "./PasswordField";
import SocialLogins from "./SocialLogins";
import { useNavigate } from "react-router-dom";
import { sendVerificationEmail } from "@/features/utils/email";
import { getPasswordStrengthLevel } from "@/features/utils/passwordUtils";
import { ThemeButton } from "@/features/play/components/ThemeButton";

const STRENGTH_CLASSES = [
	"",
	"[--strength-color:#b91c1c]",
	"[--strength-color:#92400e]",
	"[--strength-color:#2563eb]",
	"[--strength-color:#047857]",
	"[--strength-color:#0aa87b]",
] as const;

const IMG_BASE = (import.meta.env.VITE_IMAGE_MINIO as string | undefined) || "/img";

type Props = {
	mode: "signin" | "signup";
};

export default function AuthForm({ mode }: Props) {
	const { t } = useTranslation();
	const label = mode === "signin" ? t("auth.signIn") : t("auth.signUp");

	const { login, register, isLoading } = useAuth();
	const [error, setError] = useState<string>("");
	const [password, setPassword] = useState<string>("");
	const [isVisible, setIsVisible] = useState(false);

	const navigate = useNavigate();
	const passwordStrength = getPasswordStrengthLevel(password);

	useEffect(() => {
		const params = new URLSearchParams(window.location.search);
		const oauthError = params.get("error");
		if (oauthError) {
			const decoded = decodeURIComponent(oauthError).replace(/_/g, " ");
			const timer = setTimeout(() => {
				setError(`OAuth error: ${decoded}`);
				window.history.replaceState(
					{},
					document.title,
					window.location.pathname,
				);
			}, 0);
			return () => clearTimeout(timer);
		}
	}, []);

	const handleSignUp = async (data: Record<string, string>) => {

		if (data.password !== data.confirmPassword) {
			throw new Error("Passwords do not match");
		}

		if (passwordStrength < 4) {
			throw new Error("Password is too weak");
		}


		await register(data.email, data.password);
		await sendVerificationEmail();
	};

	const handleSignIn = async (data: Record<string, string>) => {

		await login(data.email, data.password);
	};

	const handleFormSubmit = async (e: React.FormEvent<HTMLFormElement>) => {
		e.preventDefault();
		setError("");

		const formData = new FormData(e.currentTarget);
		const data: Record<string, string> = {};
		formData.forEach((value, key) => {
			data[key] = value.toString();
		});


		try {
			if (mode === "signup") {
				await handleSignUp(data);
			} else {
				await handleSignIn(data);
			}
		} catch (error: unknown) {
			const msg =
				error instanceof Error
					? error.message
					: String(error) || "An unexpected error occurred";
			setError(msg);
		}
	};

	const STRENGTH_LABELS = [
		"none",
		"Very Weak",
		"Weak",
		"Moderate",
		"Strong",
		"Very Strong",
	] as const;

	const handleToggleMode = (newMode: "signin" | "signup") => () => {
		if (newMode === "signup") navigate("/signup");
		else navigate("/login");
	};

	const strengthClass =
		mode === "signup" ? STRENGTH_CLASSES[passwordStrength] : "";

	return (
		<div className="flex flex-col items-center flex-[2]">
			<div className="flex w-full items-center justify-center h-[50px] bg-[#0f172a] border border-[#334155]/60 rounded-[2px] px-2.5 mb-[25px] max-w-[min(80%,400px)]">
				<Button
					type="button"
					variant="ghost"
					onClick={handleToggleMode("signin")}
					className={`h-[80%] rounded-[2px]! text-white! ${
						mode === "signin"
							? "bg-blue-900! font-medium! mix-blend-screen hover:bg-[rgba(255,255,255,0.06)]!"
							: ""
					}`}
					style={
						mode === "signin"
							? {
									backgroundImage: `url("${IMG_BASE}/carte/battlefield_0.svg")`,
									backgroundSize: "200%",
									backgroundPosition: "left 20%",
									filter: "grayscale(100%) sepia(40%) hue-rotate(190deg) saturate(150%)",
								}
							: undefined
					}
				>
					<p>{t("auth.signIn")}</p>
				</Button>

				<Button
					type="button"
					variant="ghost"
					onClick={handleToggleMode("signup")}
					className={`h-[80%] rounded-[2px]! text-white! ${
						mode === "signup"
							? "bg-blue-900! font-medium! mix-blend-screen hover:bg-[rgba(255,255,255,0.06)]!"
							: ""
					}`}
					style={
						mode === "signup"
							? {
									backgroundImage: `url("${IMG_BASE}/carte/battlefield_0.svg")`,
									backgroundSize: "200%",
									backgroundPosition: "right 20%",
									filter: "grayscale(100%) sepia(40%) hue-rotate(190deg) saturate(150%)",
								}
							: undefined
					}
				>
					<p>{t("auth.signUp")}</p>
				</Button>
			</div>

			<SocialLogins />
			<form
				onSubmit={handleFormSubmit}
				className="flex w-full flex-col gap-3 max-w-[min(80%,400px)]"
			>
				{error && (
					<div className="bg-[#fee2e2] text-[#b91c1c] p-2.5 rounded-lg" role="alert" aria-atomic="true">
						<h3>Error:</h3>
						<p>{error}</p>
					</div>
				)}

				<input
					className="p-2.5 border border-[#334155]/60 bg-[#0f172a]/70 rounded-[2px] text-white focus:outline-none! focus:border-[#60a5fa] focus:border-2"
					name="email"
					type="email"
					placeholder={t("auth.email")}
					required
				/>

				<PasswordField
					name="password"
					placeholder={t("auth.password")}
					onChange={(e) => setPassword(e.target.value)}
					isVisible={isVisible}
					toggleVisibility={() => setIsVisible(!isVisible)}
					strengthClass={strengthClass}
				/>

				{mode === "signup" ? (
					<PasswordField
						name="confirmPassword"
						placeholder={t("auth.confirmPassword")}
						isVisible={isVisible}
						toggleVisibility={() => setIsVisible(!isVisible)}
						strengthClass={strengthClass}
					/>
				) : (
					<Button
						type="button"
						variant="ghost"
							className="text-blue-300! font-bold! justify-end! h-[50px] text-sm hover:text-white!"
							onClick={() => navigate("/forgot-password")}
					>
						{t("auth.forgotPassword")}
					</Button>
				)}

				{mode === "signup" && password && (
					<div className="p-2.5 rounded-lg text-sm text-white">
						<p>
							{t("auth.passwordStrength")}:{" "}
							<span className={`${strengthClass} [color:var(--strength-color)]`}>
								{STRENGTH_LABELS[passwordStrength]}
							</span>
						</p>
					</div>
				)}
				<ThemeButton
					type="submit"
					disabled={isLoading}
					texturePosition="center 20%"
					textureZoom={100}
					className="mt-2.5 h-[50px] w-full p-[9px] uppercase"
				>
					{isLoading ? t("auth.loading") : label}
				</ThemeButton>
			</form>
		</div>
	);
}
