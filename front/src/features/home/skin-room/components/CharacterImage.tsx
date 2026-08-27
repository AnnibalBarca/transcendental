import type { EquippedItems } from "@/features/home/skin-room/context/CosmeticContext";
import { CHARACTER_IMAGE } from "@/features/home/skin-room/components/skinRoomStyles";

const linkImg = `${import.meta.env.VITE_IMAGE_MINIO}/cosmetics/`;

type CharacterImageProps = {
  equippedItems: EquippedItems;
  onClick: () => void;
};

export default function CharacterImage({ equippedItems, onClick }: CharacterImageProps) {
  return (
    <svg viewBox="0 0 1000 1000" xmlns="http://www.w3.org/2000/svg" onClick={onClick} className={CHARACTER_IMAGE}>

      {equippedItems.base !== null && (
      <image id="layer-base" href={linkImg + "base/" + equippedItems.base.id + ".png"} width="1000" height="1000" />
      )}

      {equippedItems.hat !== null && (
        <image id="layer-hat" href={linkImg + "hat/" + equippedItems.hat.id + ".png"} width="1000" height="1000" />
      )}

      {equippedItems.mask !== null && (
        <image id="layer-mask" href={linkImg + "mask/" + equippedItems.mask.id + ".png"} width="1000" height="1000" />
      )}

      {equippedItems.clothes !== null && (
        <image id="layer-clothes" href={linkImg + "clothes/" + equippedItems.clothes.id + ".png"} width="1000" height="1000" />
      )}

      {equippedItems.accessory !== null && (
        <image id="layer-accessory" href={linkImg + "accessory/" + equippedItems.accessory.id + ".png"} width="1000" height="1000" />
      )}

    </svg>
  );
}
