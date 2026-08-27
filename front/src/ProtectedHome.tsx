import HomeLayout from "@/features/home/components/HomeLayout";
import { FriendProvider } from "@/features/friends/context/FriendContext";
import ShopPage from "@/pages/Shop/ShopPage";
import VestiairePage from "@/pages/Vestiaire/VestiairePage";
import HomePage from "@/pages/Home/HomePage";
import SocialPage from "@/pages/Social/SocialPage";
import SettingsPage from "@/pages/Settings/SettingsPage";

export default function ProtectedHome() {
	return (
		<FriendProvider>
			<HomeLayout>
				<ShopPage />
				<VestiairePage />
				<HomePage />
				<SocialPage />
				<SettingsPage />
			</HomeLayout>
		</FriendProvider>
	);
}
