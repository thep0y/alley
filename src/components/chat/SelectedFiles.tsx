import { For } from "solid-js";
import type { FileInfo } from "./types";

interface SelectedFilesProps {
  files: FileInfo[];
  onRemove: (fileId: string) => void;
}

const SelectedFiles = (props: SelectedFilesProps) => {
  return (
    <div class="selected-files">
      <For each={props.files}>
        {(file) => (
          <div class="file-card">
            <div class="file-icon">📎</div>
            <div class="file-info">
              <div class="file-name">{file.name}</div>
              <div class="file-size">{file.size}</div>
            </div>
            <button
              type="button"
              class="remove-file"
              onClick={() => props.onRemove(file.id)}
              title="移除文件"
            >
              ×
            </button>
          </div>
        )}
      </For>
    </div>
  );
};

export default SelectedFiles;
