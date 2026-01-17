import type { CameraInfo, CameraSelection } from "../../cv/types";
import { cameraSelectionKey } from "../../cv/utils";

type CameraSelectProps = {
  cameras: CameraInfo[];
  selected?: CameraSelection;
  onChange: (selection: CameraSelection) => void;
};

export const CameraSelect = ({ cameras, selected, onChange }: CameraSelectProps) => {
  const selectedKey = selected ? cameraSelectionKey(selected) : "";

  return (
    <div className="field">
      <label className="field__label">Camera</label>
      <select
        className="field__input"
        value={selectedKey}
        onChange={(event) => {
          const key = event.currentTarget.value;
          const camera = cameras.find((item) => cameraSelectionKey(item.id) === key);
          if (camera) {
            onChange(camera.id);
          }
        }}
      >
        <option value="" disabled>
          Select a camera
        </option>
        {cameras.map((camera) => (
          <option key={cameraSelectionKey(camera.id)} value={cameraSelectionKey(camera.id)}>
            {camera.name}
          </option>
        ))}
      </select>
    </div>
  );
};
