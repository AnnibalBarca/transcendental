import { useRef, useState, useEffect, useCallback } from "react";
import { useNavigate } from "react-router-dom";
import { useTranslation } from "react-i18next";
import AuthLayout from "@/features/auth/components/AuthLayout";
import AuthLeftSidebar from "@/features/auth/components/AuthLeftSidebar";
import { ThemeButton } from "@/features/play/components/ThemeButton";
import extractErrorMessage, { API_LOGIN } from "@/features/auth/services/authService";
import { useAuth } from "@/features/auth/hooks/useAuth";
import { sendVerificationEmail } from "@/features/utils/email";

const CODE_LENGTH = 6;

const CODE_INPUT_CLASS =
	"w-12 min-w-12 max-w-12 h-[58px] shrink-0 text-center text-2xl font-bold text-white bg-[#0f172a]/70 border-[1.5px] border-[#334155]/60 rounded-[2px] outline-none! caret-transparent box-border transition-[border-color,box-shadow,transform] duration-[0.15s] ease-[ease] focus:border-[#60a5fa] focus:shadow-[0_0_0_3px_rgba(96,165,250,0.2)] focus:[transform:translateY(-1px)] selection:bg-transparent! selection:text-inherit!";

export default function ValidateEmailPage() {
	const { t } = useTranslation();
	const { checkAuth, logout } = useAuth();
	const navigate = useNavigate();
	const [error, setError] = useState("");
	const [loading, setLoading] = useState(false);
	const [code, setCode] = useState<string[]>(Array(CODE_LENGTH).fill(""));
	const inputsRef = useRef<(HTMLInputElement | null)[]>([]);

	const [resendCooldown, setResendCooldown] = useState(10);
	const [resendLoading, setResendLoading] = useState(false);

	async function sendEmailValidationCheck(code: string): Promise<void> {
		const response = await fetch(`${API_LOGIN}/validate_email`, {
			method: "POST",
			headers: { "Content-Type": "application/json" },
			body: JSON.stringify({ code }),
			credentials: "include",
		});

		if (!response.ok) {
			const msg = await extractErrorMessage(
				response,
				"Failed to validate email",
			);
			throw new Error(msg);
		}
	}

	async function resendValidationEmail(): Promise<void> {
		try {
			await sendVerificationEmail();
			await checkAuth();
		} catch (err: unknown) {
		}
	}

	const handleSubmit = async (e: React.FormEvent) => {
		e.preventDefault();
		setError("");

		const fullCode = code.join("");
		if (fullCode.length !== CODE_LENGTH) {
			setError(t("password.codeLengthError"));
			return;
		}

		setLoading(true);

		try {
			await sendEmailValidationCheck(fullCode);
			await checkAuth();
		} catch (err: unknown) {
			const msg =
				err instanceof Error
					? err.message
					: String(err) || "An unexpected error occurred";
			setError(msg);
		} finally {
			setLoading(false);
		}
	};

	const handleResend = useCallback(async () => {
		if (resendCooldown > 0 || resendLoading) return;

		setResendLoading(true);
		setError("");
		try {
			await resendValidationEmail();
			setResendCooldown(60);
		} catch (err: unknown) {
			const msg =
				err instanceof Error
					? err.message
					: String(err) || t("password.resendCode");
			setError(msg);
		} finally {
			setResendLoading(false);
		}
	}, [resendCooldown, resendLoading, t]);

	useEffect(() => {
		if (resendCooldown <= 0) return;
		const interval = setInterval(() => {
			setResendCooldown((prev) => Math.max(prev - 1, 0));
		}, 1000);
		return () => clearInterval(interval);
	}, [resendCooldown]);

	const handleChange = (index: number, value: string) => {
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

	const handleBackToLogin = async () => {
		try {
			await logout();
		} catch (err: unknown) {
		}
		navigate("/login", { replace: true });
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
					<div className="flex w-full items-center justify-center h-[50px] bg-[#0f172a]/70 border border-[#334155]/60 rounded-[2px] px-2.5 mb-[25px] max-w-[min(80%,400px)]">
						<h2 className="text-[20px] font-bold text-white m-0">{t("password.validateTitle")}</h2>
					</div>
					<p className="text-sm text-[#94a3b8] text-center mb-5 max-w-[min(80%,400px)]">
						{t("password.validateSubtitle")}
					</p>
					<button
						type="button"
						onClick={handleBackToLogin}
						className="mb-3 flex items-center gap-2 bg-transparent border-none text-sm font-semibold cursor-pointer text-[#94a3b8] transition-colors duration-[0.15s] ease-[ease] enabled:hover:text-[#60a5fa]"
					>
						<span aria-hidden="true">&larr;</span>
						{t("password.backToLogin")}
					</button>
					<form onSubmit={handleSubmit} className="flex w-full flex-col gap-3 max-w-[min(80%,400px)]">
						{error && (
							<div className="bg-[#fee2e2] text-[#b91c1c] p-2.5 rounded-lg" role="alert" aria-atomic="true">
								<h3>{t("common.error")}:</h3>
								<p>{error}</p>
							</div>
						)}
						<div className="flex flex-row gap-3 justify-center items-center mt-2 mb-6 w-full">
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
									onChange={(e) => handleChange(index, e.target.value)}
									onKeyDown={(e) => handleKeyDown(index, e)}
									onFocus={handleFocus}
									onPaste={index === 0 ? handlePaste : undefined}
									autoFocus={index === 0}
								/>
							))}
						</div>
						<ThemeButton
							type="submit"
							disabled={loading || code.join("").length !== CODE_LENGTH}
							texturePosition="center 98%"
							textureZoom={130}
							className="mt-2.5 h-[50px] w-full p-[9px] uppercase"
						>
							{loading ? t("password.verifying") : t("password.verify")}
						</ThemeButton>
						<button
							type="button"
							className="mt-3 bg-transparent border-none text-sm font-semibold cursor-pointer text-[#60a5fa] transition-[color,opacity] duration-[0.15s] ease-[ease] disabled:text-[#64748b] disabled:cursor-not-allowed disabled:opacity-70 enabled:hover:text-white enabled:hover:underline"
							disabled={resendCooldown > 0 || resendLoading}
							onClick={handleResend}
						>
							{resendCooldown > 0
								? t("password.resendCodeCountdown", { countdown: resendCooldown })
								: resendLoading
									? t("password.sending")
									: t("password.resendCode")}
						</button>
					</form>
				</div>
			</div>
		</AuthLayout>
	);
}