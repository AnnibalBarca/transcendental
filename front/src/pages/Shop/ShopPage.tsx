import { SliderSection } from "@/features/play/components/SliderSection";
import { ShopView } from "@/features/shop/components/ShopView";

// Route target for /shop (see the router config). Just wires ShopView
// (the actual shop logic/state) into the app's shared horizontal-slider
// page shell used by every "play" section screen.
export default function ShopPage() {
	return (
		<SliderSection className="pb-20 lg:pb-0">
			<ShopView />
		</SliderSection>
	);
}
