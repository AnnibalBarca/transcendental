import type { Item, SlotsType } from "@/features/home/skin-room/context/CosmeticContext";
import { ITEM_LIST, itemTile, TILE_IMAGE } from "@/features/home/skin-room/components/skinRoomStyles";

const nothingImg = `${import.meta.env.VITE_IMAGE_MINIO}/assets/nothing.svg`;
const linkImg = `${import.meta.env.VITE_IMAGE_MINIO}/cosmetics/`;

type ItemListProps = {
  items: Item[];
  slotType: SlotsType;
  equippedItemId: number | undefined;
  onSelectItem: (item: Item) => void;
};

// An item with the Id === 0 used to "unequip" or while you don't have a skin, then the items pulled from the inventory in db
export default function ItemList({ items, onSelectItem, equippedItemId, slotType }: ItemListProps) {
  return (
    <ul className={ITEM_LIST}>
      {slotType !== 'base' && (
        <li className={itemTile(equippedItemId === 0 || equippedItemId === undefined)} key={0} onClick={() => onSelectItem({id: 0, name: "nothing", image: "", type: slotType})}>
          <img className={TILE_IMAGE} src={nothingImg} alt="nothing" />
        </li>
      )}

      {items?.map((item) => (
        <li className={itemTile(item.id === equippedItemId)} key={item.id} onClick={() => onSelectItem(item)}>
          <img className={TILE_IMAGE} src={(linkImg + item.type + "/" + item.id + ".png")} alt={item.name} />
        </li>
      ))}
    </ul>
  );
}
