import { useState, useEffect } from "react";
import { useSearchParams, useNavigate } from "react-router-dom";
import { useTranslation } from "react-i18next";
import AuthLayout from "@/features/auth/components/AuthLayout";
import AuthLeftSidebar from "@/features/auth/components/AuthLeftSidebar";
import Button from "@/components/ui/Button/Button";
import SuccessMessage from "@/features/auth/components/SuccessMessage";
import PasswordField from "@/features/auth/components/PasswordField";
import extractErrorMessage, { API_LOGIN } from "@/features/auth/services/authService";
import { getPasswordStrengthLevel } from "@/features/utils/passwordUtils";

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

export default function ResetPasswordPage() {
	const { t } = useTranslation();
	const [searchParams] = useSearchParams();
	const navigate = useNavigate();
	const [newPassword, setNewPassword] = useState("");
	const [confirmPassword, setConfirmPassword] = useState("");

	const [showNewPassword, setShowNewPassword] = useState(false);
	const [showConfirmPassword, setShowConfirmPassword] = useState(false);

	const [error, setError] = useState("");
	const [success, setSuccess] = useState(false);
	const [loading, setLoading] = useState(false);
	const [countdown, setCountdown] = useState(5);

	const token = searchParams.get("token");

	const passwordStrength = getPasswordStrengthLevel(newPassword);
	const strengthClass = newPassword ? STRENGTH_CLASSES[passwordStrength] : "";

	useEffect(() => {
		if (!token) {
			setError(t("password.invalidToken"));
		}
	}, [token, t]);

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

	async function resetPasswordSubmit(
		token: string,
		newPassword: string,
	): Promise<void> {
		const response = await fetch(`${API_LOGIN}/reset_password`, {
			method: "POST",
			headers: { "Content-Type": "application/json" },
			body: JSON.stringify({ token, new_password: newPassword }),
			credentials: "include",
		});

		if (!response.ok) {
			const msg = await extractErrorMessage(response, "Failed to reset password");
			throw new Error(msg);
		}
	}

	const handleSubmit = async (e: React.FormEvent) => {
		e.preventDefault();
		setError("");

		if (!token) {
			setError(t("password.invalidToken"));
			return;
		}

		const pwd = newPassword.trim();
		const confirmPwd = confirmPassword.trim();

		if (!pwd) {
			setError(t("password.passwordRequired"));
			return;
		}

		if (pwd !== confirmPwd) {
			setError(t("password.passwordsDoNotMatch"));
			return;
		}

		if (pwd.length < 8 || pwd.length > 255) {
			setError(t("password.passwordLength"));
			return;
		}

		if (passwordStrength < 4) {
			setError(t("password.passwordTooWeak"));
			return;
		}

		setLoading(true);

		try {
			await resetPasswordSubmit(token, pwd);
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
							icon={<img src={`${import.meta.env.VITE_IMAGE_MINIO}/assets/success_icon.svg`} alt="success icon" />}
							title={t("password.resetSuccessTitle")}
							message={t("password.resetSuccessMessage")}
							countdown={countdown}
							buttonText={t("password.goToLogin")}
							onButtonClick={() => navigate("/login")}
						/>
					) : (
						<>
							<div className="flex w-full items-center justify-center h-[50px] bg-[#101522] border border-[#0b3140] rounded-lg px-2.5 mb-[25px] max-w-[min(80%,400px)]">
								<h2 className="text-[20px] font-bold text-white m-0" style={{ color: "#1e293b" }}>{t("password.resetTitle")}</h2>
							</div>

							<p className="text-sm text-[#94a3b8] text-center mb-5 max-w-[min(80%,400px)]">
								{t("password.resetSubtitle")}
							</p>

							<form onSubmit={handleSubmit} className="flex w-full flex-col gap-3 max-w-[min(80%,400px)]">
								{error && (
									<div className="bg-[#fee2e2] text-[#b91c1c] p-2.5 rounded-lg" role="alert" aria-atomic="true">
										<h3>{t("common.error")}:</h3>
										<p>{error}</p>
									</div>
								)}

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
									placeholder={t("password.confirmPasswordPlaceholder")}
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
									<p style={{ color: "#64748b", fontSize: "13px", marginTop: "-10px", marginBottom: "10px" }}>
										{t("password.passwordsNotIdentical")}
									</p>
								)}

								<Button
									type="submit"
									variant="secondary"
									className="mt-2.5 p-[9px] bg-[#06b6d4]! font-bold! uppercase h-[50px] hover:bg-[rgba(255,255,255,0.12)]!"
									disabled={loading}
								>
									{loading ? t("password.resetting") : t("password.resetPassword")}
								</Button>

								<Button
									type="button"
									variant="secondary"
									className="mt-2.5 p-[9px] bg-[#06b6d4]! font-bold! uppercase h-[50px] hover:bg-[rgba(255,255,255,0.12)]!"
									onClick={() => navigate("/login")}
								>
									{t("password.backToLogin")}
								</Button>
							</form>
						</>
					)}
				</div>
			</div>
		</AuthLayout>
	);
}