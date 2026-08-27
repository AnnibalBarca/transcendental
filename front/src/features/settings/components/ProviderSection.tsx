import { useContext, useState } from "react";
import { createPortal } from "react-dom";
import { useTranslation } from "react-i18next";
import { useNavigate } from "react-router-dom";
import { AuthContext } from "@/features/auth/context/authContext";
import { toast } from "@/components/ui/toast";
import { ThemeButton } from "@/features/play/components/ThemeButton";
import { X } from "lucide-react";
import { useGoogleAuth } from "@/features/auth/hooks/useGoogleAuth";
import extractErrorMessage, { API_LOGIN } from "@/features/auth/services/authService";
import {
	SETTINGS_SECTION,
	SETTINGS_SECTION_FULL,
} from "./styles/settingsStyles";

async function switchProviderToGoogle(code: string): Promise<void> {
	const response = await fetch(`${API_LOGIN}/change_provider/google/switch`, {
		method: "POST",
		headers: {
			"Content-Type": "application/json",
		},
		body: JSON.stringify({ code }),
		credentials: "include",
	});

	if (!response.ok) {
		const msg = await extractErrorMessage(response, "Failed to switch provider to Google");
		throw new Error(msg);
	}
}

export default function ProviderSection() {
	const { t } = useTranslation();
	const navigate = useNavigate();
	const auth = useContext(AuthContext);
	const [isModalOpen, setIsModalOpen] = useState(false);

	if (!auth) {
		return null;
	}

	const { user, checkAuth } = auth;

	const { requestCode: requestGoogleCode } = useGoogleAuth(async (code: string) => {
		try {
			await switchProviderToGoogle(code);
			await checkAuth();
			navigate(0);
		} catch (error) {
			toast.add(
				{
					title: t("settings.providerSwitchFailed"),
					description: error instanceof Error ? error.message : undefined,
					type: "error",
				}
			)
		}
	});

	const handleChangePassword = () => {
		navigate("/change-password");
  };

  const handleClick = () => {
    checkAuth();
		navigate("/play");
  };

	const handleSelectProvider = (selectedProvider: string) => {
		setIsModalOpen(false);
		if (selectedProvider === "email") {
			navigate("/connect-email-provider");
		} else if (selectedProvider === "google") {
			requestGoogleCode();
		} else if (selectedProvider === "42") {
		  window.location.href = "/api/auth/42/login";
		}
	};

	if (!user) return null;

	const provider = user.auth_provider || "email";

	return (
		<div className={`${SETTINGS_SECTION} w-full ${SETTINGS_SECTION_FULL} ${provider !== "email" ? "mb-0! pb-0!" : ""}`}>
			<div className="mb-4">
				<div className="flex items-center justify-between gap-4 rounded-[2px] border border-[#334155]/60 bg-[#0f172a]/70 p-4 px-5">
					<div className="flex min-w-0 flex-1 items-center gap-3.5">
						{provider === "google" && (
							<img className="h-8 w-8 shrink-0 object-contain" src={`${import.meta.env.VITE_IMAGE_MINIO}/assets/Google_Favicon_2025.svg`} alt="Google" />
						)}
						{provider === "42" && (
							<img className="h-8 w-8 shrink-0 object-contain" src={`${import.meta.env.VITE_IMAGE_MINIO}/assets/42-Final-sigle-seul.svg`} alt="42" />
						)}
						{provider === "email" && (
							<img className="h-8 w-8 shrink-0 object-contain" src={`${import.meta.env.VITE_IMAGE_MINIO}/assets/email_icon.svg`} alt="Email" />
						)}
						<div className="flex min-w-0 flex-1 flex-col gap-0.5">
							<span className="text-[clamp(0.6rem,2.5vw,0.95rem)] font-semibold text-[#f8fafc]">
								{t("settings.connectedVia")} <span className="capitalize">{provider}</span>
							</span>
							<span className="truncate text-[clamp(0.44rem,2vw,0.85rem)] whitespace-nowrap text-[#94a3b8]">{user.email}</span>
						</div>
					</div>
					<ThemeButton
						type="button"
						texturePosition="center 98%"
						textureZoom={130}
						className="shrink-0 px-3.5 py-1.5 text-[clamp(0.8rem,2vw,0.875rem)]"
						onClick={() => setIsModalOpen(true)}
					>
						{t("settings.change")}
					</ThemeButton>
				</div>
			</div>

			{provider === "email" && (
				<ThemeButton
					type="button"
					texturePosition="center 98%"
					textureZoom={130}
					onClick={handleChangePassword}
					className="mt-auto h-[50px] w-full uppercase md:w-fit md:px-6"
				>
					{t("settings.changePassword")}
				</ThemeButton>
			)}

			{isModalOpen &&
				createPortal(
					<div className="fixed inset-0 z-[99999] flex h-screen w-screen items-center justify-center bg-black/75 p-4 backdrop-blur-[4px]" onClick={() => setIsModalOpen(false)}>
						<div
							className="flex w-full max-w-[400px] flex-col gap-4 rounded-[2px] border border-[#334155]/60 bg-[#0f172a] p-6 shadow-[0_20px_25px_-5px_rgba(0,0,0,0.5)]"
							onClick={(e) => e.stopPropagation()}
						>
							<div className="flex items-center justify-between">
								<h3 className="m-0 text-lg font-semibold text-[#f8fafc]">{t("settings.changeProvider")}</h3>
								<button
									className="flex cursor-pointer items-center justify-center border-none bg-transparent text-[#94a3b8] transition-colors hover:text-[#f8fafc]"
									onClick={() => setIsModalOpen(false)}
									type="button"
								>
									<X size={20} />
								</button>
							</div>

							<p className="m-0 text-sm text-[#94a3b8]">{t("settings.selectProvider")}</p>

							<div className="mt-0.5 flex flex-col gap-3">
								<button
									className="flex w-full cursor-pointer items-center gap-4 rounded-[2px] border border-[#334155]/60 bg-[#0f172a]/70 p-3.5 text-left text-[0.95rem] font-medium text-[#f8fafc] transition-colors duration-150 hover:border-[#60a5fa]/60 hover:bg-[#1e293b]"
									onClick={() => handleSelectProvider("google")}
									type="button"
								>
									<img src={`${import.meta.env.VITE_IMAGE_MINIO}/assets/Google_Favicon_2025.svg`} alt="Google" className="h-8 w-8 shrink-0 object-contain" />
									<span>{t("settings.google")}</span>
								</button>

								<button
									className="flex w-full cursor-pointer items-center gap-4 rounded-[2px] border border-[#334155]/60 bg-[#0f172a]/70 p-3.5 text-left text-[0.95rem] font-medium text-[#f8fafc] transition-colors duration-150 hover:border-[#60a5fa]/60 hover:bg-[#1e293b]"
									onClick={() => handleSelectProvider("42")}
									type="button"
								>
									<img src={`${import.meta.env.VITE_IMAGE_MINIO}/assets/42-Final-sigle-seul.svg`} alt="42" className="h-8 w-8 shrink-0 object-contain" />
									<span>{t("settings.fortyTwo")}</span>
								</button>

								<button
									className="flex w-full cursor-pointer items-center gap-4 rounded-[2px] border border-[#334155]/60 bg-[#0f172a]/70 p-3.5 text-left text-[0.95rem] font-medium text-[#f8fafc] transition-colors duration-150 hover:border-[#60a5fa]/60 hover:bg-[#1e293b]"
									onClick={() => handleSelectProvider("email")}
									type="button"
								>
									<img src={`${import.meta.env.VITE_IMAGE_MINIO}/assets/email_icon.svg`} alt="Email" className="h-8 w-8 shrink-0 object-contain" />
									<span>{t("settings.email")}</span>
								</button>
							</div>

							<button
								className="mt-0.5 w-full cursor-pointer border-none bg-transparent p-2 text-center text-sm text-[#94a3b8] transition-colors hover:text-[#f8fafc]"
								onClick={() => handleClick()}
								type="button"
							>
								{t("settings.cancel")}
							</button>
						</div>
					</div>,
					document.body,
				)}
		</div>
	);
}
