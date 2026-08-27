import { useState } from "react";
import { useTranslation } from "react-i18next";
import AuthLayout from "@/features/auth/components/AuthLayout";
import AuthLeftSidebar from "@/features/auth/components/AuthLeftSidebar";
import SuccessMessage from "@/features/auth/components/SuccessMessage";
import { ThemeButton } from "@/features/play/components/ThemeButton";
import { useNavigate } from "react-router-dom";
import { sendPasswordReset } from "@/features/utils/email";

export default function ForgotPasswordPage() {
	const { t } = useTranslation();
	const [email, setEmail] = useState("");
	const [error, setError] = useState("");
	const [success, setSuccess] = useState(false);
	const [loading, setLoading] = useState(false);
	const navigate = useNavigate();

	async function sendResetEmail(email: string): Promise<void> {
		try {
			await sendPasswordReset(email);
		} catch (err) {
		}
	}

	const handleSubmit = async (e: React.FormEvent) => {
		e.preventDefault();
		setError("");

		const trimmed = email.trim();
		if (trimmed.length < 3 || trimmed.length > 255) {
			setError(t("password.emailLengthError"));
			return;
		}

		setLoading(true);

		try {
			await sendResetEmail(trimmed);
			setSuccess(true);
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

	return (
		<AuthLayout>
			<div className="flex flex-row items-center w-full">
				<AuthLeftSidebar />

				<div className="flex flex-col items-center flex-[2]">
					{success ? (
						<SuccessMessage
							icon={<img src={`${import.meta.env.VITE_IMAGE_MINIO}/assets/email_icon.svg`} alt="email icon" />}
							title={t("password.emailSentTitle")}
							message={t("password.emailSentMessage")}
							showCountdown={false}
							buttonText={t("password.goToLogin")}
							onButtonClick={() => navigate("/login")}
						/>
					) : (
						<>
							<div className="flex w-full items-center justify-center h-[50px] bg-[#0f172a]/70 border border-[#334155]/60 rounded-[2px] px-2.5 mb-[25px] max-w-[min(80%,400px)]">
								<h2 className="text-[20px] font-bold text-white m-0">{t("password.forgotTitle")}</h2>
							</div>

							<p className="text-sm text-[#94a3b8] text-center mb-5 max-w-[min(80%,400px)]">
								{t("password.forgotSubtitle")}
							</p>

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
									minLength={3}
									maxLength={255}
								/>

								<ThemeButton
									type="submit"
									disabled={loading}
									texturePosition="center 98%"
									textureZoom={130}
									className="mt-2.5 h-[50px] w-full p-[9px] uppercase"
								>
									{loading ? t("password.sending") : t("password.submit")}
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