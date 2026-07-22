import "../advanced/AdvancedPanel.css";
import type { Settings } from "../../../store";
import FailsafeSection from "./FailsafeSection";
import CustomStopZoneSection from "./CustomStopZoneSection";

// ponytail: ZonesPanel is functional (FailsafeSection + CustomStopZoneSection).
// ponytail: single custom stop zone; list-of-zones if users need multiple

interface Props {
  settings: Settings;
  update: (patch: Partial<Settings>) => void;
  showInfo: boolean;
}

export default function ZonesPanel({ settings, update, showInfo }: Props) {
  return (
    <div className="adv-panel adv-panel-text">
      <div className="adv-row">
        <FailsafeSection
          settings={settings}
          update={update}
          showInfo={showInfo}
        />
      </div>
      <div className="adv-columns">
        <div className="adv-col">
          <CustomStopZoneSection
            settings={settings}
            update={update}
            showInfo={showInfo}
          />
        </div>
      </div>
    </div>
  );
}
