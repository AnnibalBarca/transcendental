import { FriendView } from "@/features/play/components/FriendView";
import { SliderSection } from "@/features/play/components/SliderSection";

export default function SocialPage() {
	return (
		<SliderSection className="pb-20 lg:pb-0">
			<FriendView />
		</SliderSection>
	);
}
