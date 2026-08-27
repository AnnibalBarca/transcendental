import { useState, useContext } from "react";
import { useTranslation } from "react-i18next";
import { ThemeButton } from "@/features/play/components/ThemeButton";
import extractErrorMessage, { API_LOGIN } from "@/features/auth/services/authService";
import { AuthContext } from "@/features/auth/context/authContext";
import { SETTINGS_LABEL, SETTINGS_SECTION, SETTINGS_SECTION_DANGER } from "./styles/settingsStyles";

const IMG_BASE = (import.meta.env.VITE_IMAGE_MINIO as string | undefined) || "/img";

export default function DeleteAccountSection() {
	const { t } = useTranslation();
	const [confirmDelete, setConfirmDelete] = useState(false);

	const auth = useContext(AuthContext);

	if (!auth) {
		throw new Error("DeleteAccountSection must be used inside an AuthProvider");
	}

	const { checkAuth } = auth;

	const handleDelete = async () => {
		if (!confirmDelete) {
			setConfirmDelete(true);
			return;
		}

		const response = await fetch(`${API_LOGIN}/delete_user`, {
			method: "DELETE",
			headers: { "Content-Type": "application/json" },
			credentials: "include",
		});

		if (!response.ok) {
			const msg = await extractErrorMessage(response, "Failed to delete account");
			throw new Error(msg);
		}

		await checkAuth();

	};

	return (
		<div className={SETTINGS_SECTION + " " + SETTINGS_SECTION_DANGER}>
			<ThemeButton
				tone="red"
				type="button"
				className="h-[50px] w-full p-[9px] uppercase"
				texture={`url("${IMG_BASE}/carte/desperate_rescue_0.svg")`}
				texturePosition="center 50%"
				textureZoom={70}
				onClick={handleDelete}
			>
				{confirmDelete ? t("settings.confirmDelete") : t("settings.deleteAccount")}
			</ThemeButton>
		</div>
	);
}
