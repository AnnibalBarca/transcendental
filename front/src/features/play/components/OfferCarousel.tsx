import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import useEmblaCarousel from "embla-carousel-react";
import Autoplay from "embla-carousel-autoplay";
import {
	offerBadge,
	offerCarousel,
	offerDot,
	offerPriceValue,
	offerSlide,
	OFFER_CONTENT,
	OFFER_DOTS,
	OFFER_OVERLAY,
	OFFER_PRICE,
	OFFER_PRICE_CURRENCY,
	OFFER_TITLE,
	OFFER_TRACK,
	OFFER_VIEWPORT,
	OFFER_WRAPPER,
	type UrgencyLevel,
} from "./offerStyles";

export interface OfferItem {
	id: string;
	backgroundImage: string;
	price: number;
	timeRemaining: string;
	expiresAt?: number;
	title?: string;
}

function parseTimeString(timeStr: string): number {
	const normalized = timeStr.toLowerCase().trim().replace(/s$/, "");
	const parseLocalizedFloat = (value: string) =>
		parseFloat(value.replace(",", "."));

	const daysMatch = normalized.match(/(\d+(?:[.,]\d+)?)\s*(?:jour|j)/);
	if (daysMatch) return parseLocalizedFloat(daysMatch[1]) * 24;

	const hoursMatch = normalized.match(/(\d+(?:[.,]\d+)?)\s*(?:heure|h)/);
	if (hoursMatch) return parseLocalizedFloat(hoursMatch[1]);

	const minsMatch = normalized.match(/(\d+(?:[.,]\d+)?)\s*(?:minute|min|m)/);
	if (minsMatch) return parseLocalizedFloat(minsMatch[1]) / 60;

	return Infinity;
}

function getUrgencyLevel(hours: number): UrgencyLevel {
	if (hours <= 1) return "expiring";
	if (hours <= 12) return "urgent";
	if (hours <= 24) return "critical";
	if (hours <= 48) return "warning";
	return "safe";
}

function getUrgencyFromOffer(offer: OfferItem): UrgencyLevel {
	if (offer.expiresAt) {
		const hoursLeft = (offer.expiresAt - Date.now()) / 1000 / 60 / 60;
		return getUrgencyLevel(hoursLeft);
	}
	return getUrgencyLevel(parseTimeString(offer.timeRemaining));
}

function getUrgencyLabel(level: UrgencyLevel): string {
	switch (level) {
		case "expiring":
			return "Expire dans l'heure";
		case "urgent":
			return "Expire aujourd'hui";
		case "critical":
			return "Dernières heures";
		case "warning":
			return "Bientôt indisponible";
		default:
			return "";
	}
}

interface OfferCarouselProps {
	offers: OfferItem[];
	autoPlayInterval?: number;
	onOfferClick?: (offer: OfferItem) => void;
}

export function OfferCarousel({
	offers,
	autoPlayInterval = 2000,
	onOfferClick,
}: OfferCarouselProps) {
	const wrapperRef = useRef<HTMLDivElement>(null);
	const [activeIndex, setActiveIndex] = useState(0);
	const [emblaRef, emblaApi] = useEmblaCarousel({ loop: offers.length > 1 }, [
		Autoplay({ delay: autoPlayInterval, stopOnInteraction: false }),
	]);

	const goTo = useCallback(
		(index: number) => {
			emblaApi?.scrollTo(index);
		},
		[emblaApi],
	);

	useEffect(() => {
		if (!emblaApi) return;

		const onSelect = () => setActiveIndex(emblaApi.selectedScrollSnap());
		emblaApi.on("select", onSelect);
		return () => {
			emblaApi.off("select", onSelect);
		};
	}, [emblaApi]);

	useEffect(() => {
		const el = wrapperRef.current;
		if (!el) return;

		const stopPropagation = (e: Event) => e.stopPropagation();
		el.addEventListener("pointerdown", stopPropagation, { passive: true });
		el.addEventListener("pointerup", stopPropagation, { passive: true });
		el.addEventListener("pointercancel", stopPropagation, { passive: true });
		el.addEventListener("touchstart", stopPropagation, { passive: true });
		el.addEventListener("touchend", stopPropagation, { passive: true });
		el.addEventListener("touchcancel", stopPropagation, { passive: true });

		return () => {
			el.removeEventListener("pointerdown", stopPropagation);
			el.removeEventListener("pointerup", stopPropagation);
			el.removeEventListener("pointercancel", stopPropagation);
			el.removeEventListener("touchstart", stopPropagation);
			el.removeEventListener("touchend", stopPropagation);
			el.removeEventListener("touchcancel", stopPropagation);
		};
	}, []);

	const activeUrgency = useMemo(
		() => {
			if (offers.length === 0) return "safe";
			const safeIndex = Math.min(activeIndex, offers.length - 1);
			return getUrgencyFromOffer(offers[safeIndex]);
		},
		[offers, activeIndex],
	);

	if (offers.length === 0) return null;

	return (
		<div ref={wrapperRef} className={OFFER_WRAPPER}>
			<div className={offerCarousel(activeUrgency)}>
				<div className={OFFER_DOTS}>
				{offers.map((_, index) => (
					<button
						key={index}
						className={offerDot(index === activeIndex)}
						onClick={() => goTo(index)}
						aria-label={`Offre ${index + 1}`}
						type="button"
					/>
				))}
			</div>

			{activeUrgency !== "safe" && (
				<div className={offerBadge(activeUrgency)}>
					{getUrgencyLabel(activeUrgency)}
				</div>
			)}

			<div className={OFFER_VIEWPORT} ref={emblaRef}>
				<div className={OFFER_TRACK}>
					{offers.map((offer) => {
						return (
							<div
								key={offer.id}
								className={offerSlide(!!onOfferClick)}
								style={{
									backgroundImage: `url(${offer.backgroundImage})`,
								}}
								onClick={() => onOfferClick?.(offer)}
							>
								<div className={OFFER_OVERLAY} />
								<div className={OFFER_CONTENT}>
									<div className="flex items-end justify-between gap-3">
										{offer.title && (
											<h3 className={OFFER_TITLE}>{offer.title}</h3>
										)}
										<div className={OFFER_PRICE}>
											<span className={offerPriceValue(activeUrgency)}>
												{offer.price}
											</span>
											<span className={OFFER_PRICE_CURRENCY}>crédits</span>
										</div>
									</div>
								</div>
							</div>
						);
					})}
				</div>
			</div>
		</div>
		</div>
	);
}
