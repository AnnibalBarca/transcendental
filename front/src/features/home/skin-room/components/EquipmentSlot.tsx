import type { SlotsType, Item } from "@/features/home/skin-room/context/CosmeticContext";
import { equipmentSlot, TILE_IMAGE } from "@/features/home/skin-room/components/skinRoomStyles";

const nothingImg = `${import.meta.env.VITE_IMAGE_MINIO}/assets/nothing.svg`;
const linkImg = `${import.meta.env.VITE_IMAGE_MINIO}/cosmetics/`;

type EquipmentSlotProps = {
  slot: SlotsType;
  isActive: boolean;
  onClick: () => void;
  equippedItem: Item | null;
};

export default function EquipmentSlot({ slot, isActive, onClick, equippedItem }: EquipmentSlotProps) {
  return (
    <button className={equipmentSlot(isActive)} onClick={onClick}>
      {equippedItem !== null &&
        <img className={TILE_IMAGE} src={(linkImg + slot + "/" + (equippedItem.id) + ".png") || nothingImg} alt={slot} />
      }
      {equippedItem == null &&
        <img className={TILE_IMAGE} src={nothingImg} alt={slot} />
      }
    </button>
  );
}
