import { useState } from "react";
import { useTranslation } from "react-i18next";
import { useSettings } from "../hooks/useSettings";
import { useAuth } from "@/features/auth/hooks/useAuth";
import { toast } from "@/components/ui/toast";
import { ThemeButton } from "@/features/play/components/ThemeButton";
import type { UserSettings } from "../services/settingsService";
import {
	SETTINGS_INPUT,
	SETTINGS_LABEL,
	SETTINGS_SECTION,
	SETTINGS_SECTION_FULL,
} from "./styles/settingsStyles";

function ProfileForm({
	settings,
	username,
	onSaved,
}: {
	settings: UserSettings;
	username: string;
	onSaved: () => void;
}) {
	const { t } = useTranslation();
	const { update } = useSettings();
	const [name, setName] = useState(() => settings.username ?? username);
	const [bio, setBio] = useState(() => settings.bio ?? "");
	const [message, setMessage] = useState<string | null>(null);
	const [error, setError] = useState<string | null>(null);

	const handleSave = async () => {
		setError(null);
		setMessage(null);
		try {
			if (name.trim() && name !== (settings.username ?? username)) {
				const response = await fetch(`/api/user/change-username`, {
					method: "PATCH",
					headers: { "Content-Type": "application/json" },
					body: JSON.stringify({ name: name.trim() }),
					credentials: "include",
				});
				if (!response.ok) {
					const data = await response.json().catch(() => null);
					throw new Error(data?.error ?? t("settings.profileSaveFailed"));
				}
			}
			await update({ bio: bio.trim() });
			setMessage(t("settings.profileSaved"));
			onSaved();
			toast.add(
				{
					title: t("settings.profileSaved"),
					type: "success",
				}
			)
		} catch (err) {
			setError(err instanceof Error ? err.message : t("settings.profileSaveFailed"));
			toast.add(
				{
					title: t("settings.profileSaveFailed"),
					description: err instanceof Error ? err.message : undefined,
					type: "error",
				}
			)
		}
	};

	return (
		<div className={`${SETTINGS_SECTION} ${SETTINGS_SECTION_FULL}`}>
			<label className={SETTINGS_LABEL} htmlFor="username">
				{t("settings.nickname")}
			</label>
			<input
				id="username"
				className={SETTINGS_INPUT}
				value={name}
				onChange={(e) => setName(e.target.value)}
			/>

			<label className={SETTINGS_LABEL} htmlFor="bio">
				{t("settings.bio")}
			</label>
			<textarea
				id="bio"
				className={`${SETTINGS_INPUT} min-h-20 py-2`}
				rows={3}
				value={bio}
				maxLength={1500}
				onChange={(e) => setBio(e.target.value)}
			/>

			{message && <p className="m-0 text-xs text-emerald-400">{message}</p>}
			{error && <p className="m-0 text-xs text-red-400">{error}</p>}

			<ThemeButton
				type="button"
				className="self-start px-5 py-2.5 text-sm uppercase tracking-[1px]"
				texturePosition="center 98%"
				textureZoom={130}
				onClick={handleSave}
			>
				{t("settings.saveProfile")}
			</ThemeButton>
		</div>
	);
}

export default function ProfileSection() {
	const { settings, loading } = useSettings();
	const { user } = useAuth();
	const [, setSaved] = useState(0);

	if (loading) return null;

	return (
		<ProfileForm
			key={settings?.username ?? "none"}
			settings={
				settings ?? {
					username: user?.username,
					email: user?.email,
					bio: "",
					github: "",
					discord: "",
					twitter: "",
					is_private: false,
					theme: "dark",
				}
			}
			username={user?.username ?? ""}
			onSaved={() => setSaved((n) => n + 1)}
		/>
	);
}