interface FileUploadProps {
  isMobile: boolean;
  onFileSelect: (e: Event) => void;
  fileInputRef: (el: HTMLInputElement) => void;
}

const FileUpload = (props: FileUploadProps) => {
  const inputId = props.isMobile ? "mobile-file-input" : "file-input";

  return (
    <div class={props.isMobile ? "mobile-toolbar" : "desktop-toolbar"}>
      <div class="toolbar-left">
        <input
          ref={props.fileInputRef}
          type="file"
          multiple
          onChange={props.onFileSelect}
          class="hidden-input"
          id={inputId}
        />
        <label for={inputId} class="toolbar-button" title="添加文件">
          <span class="icon">📎</span>
        </label>
      </div>
    </div>
  );
};

export default FileUpload;
