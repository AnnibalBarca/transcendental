import { useState, useEffect, useRef, useCallback } from "react";
import { useNavigate } from "react-router-dom";
import { useTranslation } from "react-i18next";
import AuthLayout from "@/features/auth/components/AuthLayout";
import AuthLeftSidebar from "@/features/auth/components/AuthLeftSidebar";
import SuccessMessage from "@/features/auth/components/SuccessMessage";
import PasswordField from "@/features/auth/components/PasswordField";
import { ThemeButton } from "@/features/play/components/ThemeButton";
import extractErrorMessage, { API_LOGIN } from "@/features/auth/services/authService";
import { getPasswordStrengthLevel } from "@/features/utils/passwordUtils";
import { sendVerificationEmailProviderChange } from "@/features/utils/email";

const CODE_LENGTH = 6;

const STRENGTH_LEVEL = [
	"none",
	"veryWeak",
	"weak",
	"moderate",
	"strong",
	"veryStrong",
] as const;

const STRENGTH_CLASSES = [
	"",
	"[--strength-color:#b91c1c]",
	"[--strength-color:#92400e]",
	"[--strength-color:#2563eb]",
	"[--strength-color:#047857]",
	"[--strength-color:#0aa87b]",
] as const;

const CODE_INPUT_CLASS =
	"w-12 min-w-12 max-w-12 h-[58px] shrink-0 text-center text-2xl font-bold text-white bg-[#0f172a]/70 border-[1.5px] border-[#334155]/60 rounded-[2px] outline-none! caret-transparent box-border transition-[border-color,box-shadow,transform] duration-[0.15s] ease-[ease] focus:border-[#60a5fa] focus:shadow-[0_0_0_3px_rgba(96,165,250,0.2)] focus:[transform:translateY(-1px)] selection:bg-transparent! selection:text-inherit!";

export default function ChangeProviderPage() {
	const { t } = useTranslation();
	const navigate = useNavigate();
	const [email, setEmail] = useState("");
	const [newPassword, setNewPassword] = useState("");
	const [confirmPassword, setConfirmPassword] = useState("");

	const [code, setCode] = useState<string[]>(Array(CODE_LENGTH).fill(""));
	const inputsRef = useRef<(HTMLInputElement | null)[]>([]);

	const [resendCooldown, setResendCooldown] = useState(0);
	const [resendLoading, setResendLoading] = useState(false);

	const [showNewPassword, setShowNewPassword] = useState(false);
	const [showConfirmPassword, setShowConfirmPassword] = useState(false);

	const [error, setError] = useState("");
	const [success, setSuccess] = useState(false);
	const [loading, setLoading] = useState(false);
	const [countdown, setCountdown] = useState(5);

	const passwordStrength = getPasswordStrengthLevel(newPassword);
	const strengthClass = newPassword ? STRENGTH_CLASSES[passwordStrength] : "";

	const isFormValid =
		email.trim().length > 0 &&
		code.join("").length === CODE_LENGTH &&
		newPassword.trim().length >= 8 &&
		newPassword.trim().length <= 255 &&
		passwordStrength >= 4 &&
		newPassword.trim() === confirmPassword.trim();

	useEffect(() => {
		if (!success) return;

		const timer = setInterval(() => {
			setCountdown((prev) => {
				if (prev <= 1) {
					navigate("/login");
					return 0;
				}
				return prev - 1;
			});
		}, 1000);

		return () => clearInterval(timer);
	}, [success, navigate]);

	useEffect(() => {
		if (resendCooldown <= 0) return;
		const interval = setInterval(() => {
			setResendCooldown((prev) => Math.max(prev - 1, 0));
		}, 1000);
		return () => clearInterval(interval);
	}, [resendCooldown]);

	const handleSendCode = useCallback(async () => {
		if (resendCooldown > 0 || resendLoading) return;

		setError("");
		const trimmedEmail = email.trim();
		if (!trimmedEmail) {
			setError(t("password.emailFirstError"));
			return;
		}

		setResendLoading(true);
		try {
			await sendVerificationEmailProviderChange(email);
			setResendCooldown(60);
		} catch (err: unknown) {
			const msg = err instanceof Error ? err.message : t("password.sendVerificationCode");
			setError(msg);
		} finally {
			setResendLoading(false);
		}
	}, [resendCooldown, resendLoading, email, t]);

	async function changeProviderSubmit(emailVal: string, codeVal: string, passwordVal: string): Promise<void> {
		const response = await fetch(`${API_LOGIN}/change_provider/email/switch`, {
			method: "POST",
			headers: { "Content-Type": "application/json" },
			body: JSON.stringify({
				email: emailVal,
				code: codeVal,
				password: passwordVal
			}),
			credentials: "include",
		});

		if (!response.ok) {
			const msg = await extractErrorMessage(response, "Failed to change provider");
			throw new Error(msg);
		}
	}

	const handleSubmit = async (e: React.FormEvent) => {
		e.preventDefault();
		setError("");

		if (!isFormValid) return;

		setLoading(true);

		try {
			await changeProviderSubmit(email.trim(), code.join(""), newPassword.trim());
			setSuccess(true);
		} catch (err: unknown) {
			const msg =
				err instanceof Error ? err.message : String(err) || "An unexpected error occurred";
			setError(msg);
		} finally {
			setLoading(false);
		}
	};

	const handleChangeCode = (index: number, value: string) => {
		const digit = value.replace(/[^0-9]/g, "").slice(-1);
		const newCode = [...code];
		newCode[index] = digit;
		setCode(newCode);

		if (digit && index < CODE_LENGTH - 1) {
			inputsRef.current[index + 1]?.focus();
		}
	};

	const handleFocus = (e: React.FocusEvent<HTMLInputElement>) => {
		e.target.select();
	};

	const handleKeyDown = (index: number, e: React.KeyboardEvent<HTMLInputElement>) => {
		if (e.key === "Backspace" && !code[index] && index > 0) {
			inputsRef.current[index - 1]?.focus();
		}
	};

	const handlePaste = (e: React.ClipboardEvent<HTMLInputElement>) => {
		e.preventDefault();
		const pasted = e.clipboardData.getData("text").replace(/[^0-9]/g, "").slice(0, CODE_LENGTH);
		if (!pasted) return;

		const newCode = Array(CODE_LENGTH).fill("");
		pasted.split("").forEach((char, i) => {
			newCode[i] = char;
		});
		setCode(newCode);
		inputsRef.current[Math.min(pasted.length, CODE_LENGTH - 1)]?.focus();
	};

	return (
		<AuthLayout>
			<div className="flex flex-row items-center w-full">
				<AuthLeftSidebar />

				<div className="flex flex-col items-center flex-[2]">
					{success ? (
						<SuccessMessage
							icon={<img src={`${import.meta.env.VITE_IMAGE_MINIO}/assets/success_icon.svg`} alt="success icon" />}
							title={t("password.providerSuccessTitle")}
							message={t("password.providerSuccessMessage")}
							countdown={countdown}
							buttonText={t("password.goBack")}
							onButtonClick={() => navigate("/play")}
						/>
					) : (
						<>
							<div className="flex w-full items-center justify-center h-[50px] bg-[#0f172a]/70 border border-[#334155]/60 rounded-[2px] px-2.5 mb-[25px] max-w-[min(80%,400px)]">
								<h2 className="m-0 text-xl font-bold text-white">{t("password.connectTitle")}</h2>
							</div>

							<form onSubmit={handleSubmit} className="flex w-full flex-col gap-3 max-w-[min(80%,400px)]">
								{error && (
									<div className="bg-[#fee2e2] text-[#b91c1c] p-2.5 rounded-lg" role="alert" aria-atomic="true">
										<h3>{t("common.error")}:</h3>
										<p>{error}</p>
									</div>
								)}

								<input
									className="p-2.5 border border-[#334155]/60 bg-[#0f172a]/70 rounded-[2px] text-white focus:outline-none! focus:border-[#60a5fa] focus:border-2"
									type="email"
									name="email"
									placeholder={t("auth.email")}
									value={email}
									onChange={(e) => setEmail(e.target.value)}
									required
								/>

									<ThemeButton
									type="button"
									texturePosition="center 98%"
									textureZoom={130}
									onClick={handleSendCode}
									disabled={resendCooldown > 0 || resendLoading || !email}
									className="mt-2 h-[42px] w-full p-0 text-sm"
								>
									{resendCooldown > 0
										? t("password.resendCodeCountdown", { countdown: resendCooldown })
										: resendLoading
											? t("password.sending")
											: t("password.sendVerificationCode")}
								</ThemeButton>

								<PasswordField
									name="new_password"
									placeholder={t("password.newPasswordPlaceholder")}
									value={newPassword}
									onChange={(e) => setNewPassword(e.target.value)}
									isVisible={showNewPassword}
									toggleVisibility={() => setShowNewPassword(!showNewPassword)}
									strengthClass={strengthClass}
									minLength={8}
									maxLength={255}
								/>

								<PasswordField
									name="confirm_password"
									placeholder={t("password.confirmNewPasswordPlaceholder")}
									value={confirmPassword}
									onChange={(e) => setConfirmPassword(e.target.value)}
									isVisible={showConfirmPassword}
									toggleVisibility={() => setShowConfirmPassword(!showConfirmPassword)}
									strengthClass={strengthClass}
									minLength={8}
									maxLength={255}
								/>

								{newPassword && (
									<div className="p-2.5 rounded-lg text-sm text-white">
										<p>
											{t("auth.passwordStrength")}:{" "}
											<span className={`${strengthClass} [color:var(--strength-color)]`}>
												{t(`password.strength.${STRENGTH_LEVEL[passwordStrength]}`)}
											</span>
										</p>
									</div>
								)}

								{confirmPassword && newPassword !== confirmPassword && (
									<p className="m-0 text-center text-base leading-relaxed text-[#ff4d4f]">
										{t("password.passwordsDoNotMatch")}.
									</p>
								)}

								<div className="flex flex-row gap-3 justify-center items-center mt-2 mb-6 w-full" style={{ marginTop: "10px" }}>
									{code.map((digit, index) => (
										<input
											key={index}
											ref={(el) => {
												inputsRef.current[index] = el;
											}}
											className={CODE_INPUT_CLASS}
											type="text"
											inputMode="numeric"
											value={digit}
											onChange={(e) => handleChangeCode(index, e.target.value)}
											onKeyDown={(e) => handleKeyDown(index, e)}
											onFocus={handleFocus}
											onPaste={index === 0 ? handlePaste : undefined}
										/>
									))}
								</div>

								<ThemeButton
									type="submit"
									texturePosition="center 98%"
									textureZoom={130}
									disabled={loading || !isFormValid}
									className="mt-2.5 h-[50px] w-full p-[9px] uppercase"
								>
									{loading ? t("password.changing") : t("password.connectEmail")}
								</ThemeButton>

								<ThemeButton
									type="button"
									texturePosition="center 98%"
									textureZoom={130}
									className="mt-2.5 h-[50px] w-full p-[9px] uppercase"
									onClick={() => navigate("/login")}
								>
									{t("common.cancel")}
								</ThemeButton>
							</form>
						</>
					)}
				</div>
			</div>
		</AuthLayout>
	);
}