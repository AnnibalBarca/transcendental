import { useState, useEffect } from "react";
import { useNavigate } from "react-router-dom";
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

export default function ChangePasswordPage() {
	const { t } = useTranslation();
	const navigate = useNavigate();
	const [currentPassword, setCurrentPassword] = useState("");
	const [newPassword, setNewPassword] = useState("");
	const [confirmPassword, setConfirmPassword] = useState("");

	const [showCurrentPassword, setShowCurrentPassword] = useState(false);
	const [showNewPassword, setShowNewPassword] = useState(false);

	const [error, setError] = useState("");
	const [success, setSuccess] = useState(false);
	const [loading, setLoading] = useState(false);
	const [countdown, setCountdown] = useState(5);

	const passwordStrength = getPasswordStrengthLevel(newPassword);
	const strengthClass = newPassword ? STRENGTH_CLASSES[passwordStrength] : "";

	const isFormValid =
		currentPassword.trim().length > 0 &&
		newPassword.trim().length >= 8 &&
		newPassword.trim().length <= 255 &&
		passwordStrength >= 4 &&
		newPassword.trim() === confirmPassword.trim() &&
		newPassword.trim() !== currentPassword.trim();

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

	async function changePasswordSubmit(oldPassword: string, newPassword: string): Promise<void> {
		const response = await fetch(`${API_LOGIN}/change_password`, {
			method: "POST",
			headers: { "Content-Type": "application/json" },
			body: JSON.stringify({ old_password: oldPassword, new_password: newPassword }),
			credentials: "include",
		});

		if (!response.ok) {
			const msg = await extractErrorMessage(response, "Failed to change password");
			throw new Error(msg);
		}
	}

	const handleSubmit = async (e: React.FormEvent) => {
		e.preventDefault();
		setError("");

		if (!isFormValid) return;

		const oldPwd = currentPassword.trim();
		const newPwd = newPassword.trim();

		setLoading(true);

		try {
			await changePasswordSubmit(oldPwd, newPwd);
			setSuccess(true);
		} catch (err: unknown) {
			const msg =
				err instanceof Error ? err.message : String(err) || "An unexpected error occurred";
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
							title={t("password.changeSuccessTitle")}
							message={t("password.changeSuccessMessage")}
							countdown={countdown}
							buttonText={t("password.goBack")}
							onButtonClick={() => navigate("/play")}
						/>
					) : (
						<>
							<div className="flex w-full items-center justify-center h-[50px] bg-[#101522] border border-[#0b3140] rounded-lg px-2.5 mb-[25px] max-w-[min(80%,400px)]">
								<h2 className="m-0 text-xl font-bold text-white">{t("password.changeTitle")}</h2>
							</div>

							<form onSubmit={handleSubmit} className="flex w-full flex-col gap-3 max-w-[min(80%,400px)]">
								{error && (
									<div className="bg-[#fee2e2] text-[#b91c1c] p-2.5 rounded-lg" role="alert" aria-atomic="true">
										<h3>{t("common.error")}:</h3>
										<p>{error}</p>
									</div>
								)}

								<PasswordField
									name="current_password"
									placeholder={t("password.currentPasswordPlaceholder")}
									value={currentPassword}
									onChange={(e) => setCurrentPassword(e.target.value)}
									isVisible={showCurrentPassword}
									toggleVisibility={() => setShowCurrentPassword(!showCurrentPassword)}
								/>

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
									isVisible={showNewPassword}
									toggleVisibility={() => setShowNewPassword(!setShowNewPassword)}
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

								{currentPassword && newPassword == currentPassword && (
									<p className="m-0 text-center text-base leading-relaxed text-[#ff4d4f]">
										{t("password.passwordSameAsCurrent")}
									</p>
								)}

								<Button
									type="submit"
									variant="secondary"
									className="mt-2.5 p-[9px] bg-[#06b6d4]! font-bold! uppercase h-[50px] hover:bg-[rgba(255,255,255,0.12)]!"
									disabled={loading || !isFormValid}
								>
									{loading ? t("password.changing") : t("password.changePassword")}
								</Button>

								<Button
									type="button"
									variant="secondary"
									className="mt-2.5 p-[9px] bg-[#06b6d4]! font-bold! uppercase h-[50px] hover:bg-[rgba(255,255,255,0.12)]!"
									onClick={() => navigate("/login")}
								>
									{t("common.cancel")}
								</Button>
							</form>
						</>
					)}
				</div>
			</div>
		</AuthLayout>
	);
}