import { useContext } from "react";
import { useTranslation } from "react-i18next";
import { toast } from "@/components/ui/toast";
import { ThemeButton } from "@/features/play/components/ThemeButton";
import { AuthContext } from "@/features/auth/context/authContext";
import { API_LOGIN } from "@/features/auth/services/authService";
import { SETTINGS_SECTION } from "./styles/settingsStyles";

const IMG_BASE = (import.meta.env.VITE_IMAGE_MINIO as string | undefined) || "/img";

export default function LogoutButton() {
	const { t } = useTranslation();
	const auth = useContext(AuthContext);

	if (!auth) {
		return null;
	}

	const { checkAuth } = auth;

	const handleLogout = async () => {
		const response = await fetch(`${API_LOGIN}/logout`, {
			method: "POST",
			headers: { "Content-Type": "application/json" },
			credentials: "include",
		});

		if (!response.ok) {
			toast.add(
				{
					title: t("settings.logoutFailed"),
					type: "error",
				}
			)
		}

		await checkAuth();

	};

	return (
		<div className={`${SETTINGS_SECTION} justify-center`}>
			<ThemeButton
				type="button"
				tone="red"
				onClick={handleLogout}
				className="h-[50px] w-full p-[9px] uppercase"
				texturePosition="center 50%"
				textureZoom={80}
				texture={`url("${IMG_BASE}/carte/ninja_0.svg")`}
			>
				{t("settings.logout")}
			</ThemeButton>
		</div>
	);
}
