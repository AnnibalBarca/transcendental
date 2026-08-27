import { useState } from "react";
import { useTranslation } from "react-i18next";
import { useAuth } from "@/features/auth/hooks/useAuth";
import AuthLayout from "@/features/auth/components/AuthLayout";
import AuthLeftSidebar from "@/features/auth/components/AuthLeftSidebar";
import { ThemeButton } from "@/features/play/components/ThemeButton";

export default function SetupPage() {
	const { t } = useTranslation();
	const { finishAccount, isLoading } = useAuth();
	const [username, setUsername] = useState("");
	const [error, setError] = useState("");

	const handleSubmit = async (e: React.FormEvent) => {
		e.preventDefault();
		setError("");

		const trimmed = username.trim();
		if (trimmed.length < 3 || trimmed.length > 255) {
			setError(t("setup.usernameError"));
			return;
		}

		try {
			await finishAccount(trimmed);
		} catch (err: unknown) {
			const msg =
				err instanceof Error
					? err.message
					: String(err) || "An unexpected error occurred";
			setError(msg);
		}
	};

	return (
		<AuthLayout>
			<div className="flex flex-row items-center w-full">
				<AuthLeftSidebar />

				<div className="flex flex-col items-center flex-[2]">
					<div className="flex w-full items-center justify-center h-[50px] bg-[#0f172a]/70 border border-[#334155]/60 rounded-[2px] px-2.5 mb-[25px] max-w-[min(80%,400px)]">
						<h2 className="text-white text-base font-semibold m-0">{t("setup.title")}</h2>
					</div>

					<p className="text-[0.9rem] text-[rgba(255,255,255,0.6)] text-center m-0 mb-5 w-full max-w-[min(80%,400px)]">
						{t("setup.subtitle")}
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
							type="text"
							name="username"
							placeholder={t("setup.usernamePlaceholder")}
							value={username}
							onChange={(e) => setUsername(e.target.value)}
							required
							minLength={3}
							maxLength={255}
						/>

						<ThemeButton
							type="submit"
							disabled={isLoading}
							texturePosition="center 98%"
							textureZoom={130}
							className="mt-2.5 h-[50px] w-full p-[9px] uppercase"
						>
							{isLoading ? t("common.loading") : t("setup.finish")}
						</ThemeButton>
					</form>
				</div>
			</div>
		</AuthLayout>
	);
}