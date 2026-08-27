import { useTranslation } from "react-i18next";
import { useAuth } from "@/features/auth/hooks/useAuth";
import { useGoogleAuth } from "@/features/auth/hooks/useGoogleAuth";
import { ThemeButton } from "@/features/play/components/ThemeButton";

export default function GoogleLogin() {
	const { t } = useTranslation();
	const { googleLogin } = useAuth();
	const { requestCode } = useGoogleAuth(googleLogin);

	return (
<ThemeButton
			type="button"
			onClick={requestCode}
			texture="/img/carte/beast_of_burden_0.svg"
			texturePosition="center 70%"
			textureZoom={100}
			className="h-[50px] w-full justify-start gap-3 rounded-[2px] pl-[15px]"
		>
			<img
				className="shrink-0"
				src={`${import.meta.env.VITE_IMAGE_MINIO}/assets/Google_Favicon_2025.svg`}
				alt="Google"
				width={30}
				height={30}
				onError={(e) => {
					e.currentTarget.style.display = "none";
				}}
			/>
			<span>{t("auth.continueWithGoogle")}</span>
		</ThemeButton>
	);
}
