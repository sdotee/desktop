import SwiftUI
import SwiftData
import UniformTypeIdentifiers
#if os(iOS)
import PhotosUI
import AVFoundation
#endif

enum LinkDisplayType: String, CaseIterable, Identifiable {
    case directLink = "Direct Link"
    case sharePage = "Share Page"
    case bbcode = "BBCode"
    case bbcodeWithLink = "BBCode w/ Link"
    case bbcodeDirectLink = "BBCode w/ Direct Link"
    case html = "HTML"
    case htmlWithLink = "HTML w/ Link"
    case htmlDirectLink = "HTML w/ Direct Link"
    case markdown = "Markdown"

    var id: String { rawValue }

    func formatted(file: UploadedFile) -> String {
        switch self {
        case .directLink:
            return file.url
        case .sharePage:
            return file.page
        case .bbcode:
            return LinkFormatter.bbcode(filename: file.filename, directURL: file.url)
        case .bbcodeWithLink:
            return LinkFormatter.bbcodeWithLink(filename: file.filename, pageURL: file.page, directURL: file.url)
        case .bbcodeDirectLink:
            return LinkFormatter.bbcodeDirectLink(filename: file.filename, directURL: file.url)
        case .html:
            return LinkFormatter.html(filename: file.filename, directURL: file.url)
        case .htmlWithLink:
            return LinkFormatter.htmlWithLink(filename: file.filename, pageURL: file.page, directURL: file.url)
        case .htmlDirectLink:
            return LinkFormatter.htmlDirectLink(filename: file.filename, directURL: file.url)
        case .markdown:
            return LinkFormatter.markdown(filename: file.filename, directURL: file.url)
        }
    }
}

struct FileUploadView: View {
    @Environment(\.modelContext) private var modelContext
    @Query(sort: \UploadedFile.createdAt, order: .reverse) private var files: [UploadedFile]
    @State private var viewModel = FileUploadViewModel()
    @State private var showFilePicker = false
    @State private var fileToDelete: UploadedFile?
    @State private var selectedFileIDs: Set<PersistentIdentifier> = []
    @State private var showBatchLinks = false
    @State private var showBatchDeleteAlert = false
    @State private var currentPage = 1
    @State private var linkDisplayType: LinkDisplayType = {
        if let saved = UserDefaults.standard.string(forKey: Constants.defaultFileLinkDisplayKey),
           let type = LinkDisplayType(rawValue: saved) {
            return type
        }
        return .sharePage
    }()

    #if os(iOS)
    @State private var showPhotoPicker = false
    @State private var selectedPhoto: PhotosPickerItem?
    @State private var showCamera = false
    #endif

    private var fileTotalPages: Int { Pagination.totalPages(for: files.count) }
    private var pagedFiles: [UploadedFile] { Pagination.page(files, page: currentPage) }

    private var selectedFiles: [UploadedFile] {
        files.filter { selectedFileIDs.contains($0.persistentModelID) }
    }

    var body: some View {
        VStack(spacing: 0) {
            // Upload area
            uploadArea

            // File list
            if files.isEmpty {
                EmptyStateView(
                    icon: "arrow.up.doc",
                    title: String(localized: "No Uploaded Files"),
                    message: String(localized: "Upload files to share them with a link.")
                )
            } else {
                List {
                    ForEach(pagedFiles) { file in
                        HStack(spacing: 8) {
                            Image(systemName: selectedFileIDs.contains(file.persistentModelID) ? "checkmark.circle.fill" : "circle")
                                .foregroundStyle(selectedFileIDs.contains(file.persistentModelID) ? Color.accentColor : .secondary.opacity(0.4))
                                .font(.body)

                            UploadedFileRow(file: file, linkDisplayType: linkDisplayType) {
                                fileToDelete = file
                            }
                        }
                        .contentShape(Rectangle())
                        .onTapGesture {
                            withAnimation(.easeInOut(duration: 0.15)) {
                                if selectedFileIDs.contains(file.persistentModelID) {
                                    selectedFileIDs.remove(file.persistentModelID)
                                } else {
                                    selectedFileIDs.insert(file.persistentModelID)
                                }
                            }
                        }
                    }

                    if fileTotalPages > 1 {
                        PaginationView(currentPage: currentPage, totalPages: fileTotalPages) { page in
                            currentPage = page
                        }
                        .frame(maxWidth: .infinity)
                        .listRowSeparator(.hidden)
                    }
                }
                #if os(iOS)
                .contentMargins(.top, 16, for: .scrollContent)
                #endif
            }
        }
        #if os(macOS)
        .dropDestination(for: URL.self) { urls, _ in
            Task {
                for url in urls {
                    guard let data = try? Data(contentsOf: url) else { continue }
                    let _ = await viewModel.uploadFile(
                        data: data,
                        filename: url.lastPathComponent,
                        context: modelContext
                    )
                }
            }
            return true
        }
        #endif
        .navigationTitle(String(localized: "File Upload"))
        .toolbar {
            #if os(iOS)
            ToolbarItem(placement: .topBarTrailing) {
                HStack(spacing: 4) {
                    Menu {
                        ForEach(LinkDisplayType.allCases) { type in
                            Button(action: { linkDisplayType = type }) {
                                if linkDisplayType == type {
                                    Label(type.rawValue, systemImage: "checkmark")
                                } else {
                                    Text(type.rawValue)
                                }
                            }
                        }
                    } label: {
                        Text(linkDisplayType.rawValue)
                            .font(.subheadline)
                            .lineLimit(1)
                    }

                    Button(action: { showBatchLinks = true }) {
                        Image(systemName: "link.badge.plus")
                    }
                    .disabled(selectedFileIDs.isEmpty)

                    Button(action: { showBatchDeleteAlert = true }) {
                        Image(systemName: "trash")
                    }
                    .tint(.red)
                    .disabled(selectedFileIDs.isEmpty)
                }
            }
            #else
            ToolbarItem(placement: .automatic) {
                Menu {
                    ForEach(LinkDisplayType.allCases) { type in
                        Button(action: { linkDisplayType = type }) {
                            if linkDisplayType == type {
                                Label(type.rawValue, systemImage: "checkmark")
                            } else {
                                Text(type.rawValue)
                            }
                        }
                    }
                } label: {
                    HStack(spacing: 4) {
                        Text(linkDisplayType.rawValue)
                        Image(systemName: "chevron.up.chevron.down")
                            .font(.caption2)
                            .foregroundStyle(.secondary)
                    }
                    .frame(minWidth: 100)
                }
                .menuStyle(.borderlessButton)
                .fixedSize()
            }
            ToolbarItem(placement: .automatic) {
                Button(action: { showBatchLinks = true }) {
                    Label(
                        selectedFileIDs.isEmpty
                            ? String(localized: "Get Links")
                            : String(localized: "Get Links (\(selectedFileIDs.count))"),
                        systemImage: "link.badge.plus"
                    )
                }
                .disabled(selectedFileIDs.isEmpty)
            }
            ToolbarItem(placement: .automatic) {
                Button(action: { showBatchDeleteAlert = true }) {
                    Label(String(localized: "Delete (\(selectedFileIDs.count))"), systemImage: "trash")
                }
                .tint(.red)
                .disabled(selectedFileIDs.isEmpty)
            }
            ToolbarItem(placement: .automatic) {
                Button(action: handlePasteUpload) {
                    Label(String(localized: "Paste"), systemImage: "doc.on.clipboard")
                }
                .keyboardShortcut("v", modifiers: .command)
                .disabled(viewModel.isLoading)
            }
            #endif
        }
        .alert(String(localized: "Delete File?"), isPresented: .init(
            get: { fileToDelete != nil },
            set: { if !$0 { fileToDelete = nil } }
        )) {
            Button(String(localized: "Cancel"), role: .cancel) {}
            Button(String(localized: "Delete"), role: .destructive) {
                if let file = fileToDelete {
                    Task {
                        let _ = await viewModel.deleteFile(file, context: modelContext)
                    }
                }
            }
        } message: {
            Text(String(localized: "This will permanently delete the file."))
        }
        .alert(String(localized: "Delete \(selectedFileIDs.count) Files?"), isPresented: $showBatchDeleteAlert) {
            Button(String(localized: "Cancel"), role: .cancel) {}
            Button(String(localized: "Delete All"), role: .destructive) {
                Task {
                    await batchDeleteSelectedFiles()
                }
            }
        } message: {
            Text(String(localized: "This will permanently delete \(selectedFileIDs.count) selected files. This action cannot be undone."))
        }
        .sheet(isPresented: $showBatchLinks) {
            BatchLinksView(files: selectedFiles, linkDisplayType: linkDisplayType) {
                selectedFileIDs.removeAll()
                showBatchLinks = false
            }
            #if os(iOS)
            .presentationDetents([.medium, .large])
            #endif
        }
        .fileImporter(
            isPresented: $showFilePicker,
            allowedContentTypes: [.data],
            allowsMultipleSelection: true
        ) { result in
            handleFilePicker(result)
        }
        #if os(iOS)
        .photosPicker(isPresented: $showPhotoPicker, selection: $selectedPhoto)
        .onChange(of: selectedPhoto) { _, newValue in
            if let item = newValue {
                handlePhotoPicker(item)
            }
        }
        .fullScreenCover(isPresented: $showCamera) {
            CameraView(onCapture: { image in
                handleCameraCapture(image)
            }, onVideoCapture: { url in
                handleVideoCapture(url)
            })
            .ignoresSafeArea()
        }
        #endif
        .toast(message: $viewModel.successMessage)
        .toast(message: $viewModel.errorMessage, isError: true)
        .task {
            await viewModel.loadDomains()
        }
    }

    @ViewBuilder
    private var uploadArea: some View {
        #if os(macOS)
        DropZoneView(isLoading: viewModel.isLoading, progress: viewModel.uploadProgress) { urls in
            Task {
                for url in urls {
                    guard let data = try? Data(contentsOf: url) else { continue }
                    let _ = await viewModel.uploadFile(
                        data: data,
                        filename: url.lastPathComponent,
                        context: modelContext
                    )
                }
            }
        } onTap: {
            showFilePicker = true
        }
        #else
        VStack(spacing: 12) {
            if viewModel.isLoading {
                VStack(spacing: 8) {
                    ProgressView(value: viewModel.uploadProgress)
                        .tint(.accentColor)
                    Text(String(localized: "Uploading... \(Int(viewModel.uploadProgress * 100))%"))
                        .font(.caption)
                        .foregroundStyle(.secondary)
                }
                .padding()
            } else {
                HStack(spacing: 24) {
                    UploadActionButton(title: String(localized: "File"), icon: "doc", tint: .accentColor) {
                        showFilePicker = true
                    }
                    UploadActionButton(title: String(localized: "Photo"), icon: "photo", tint: .orange) {
                        showPhotoPicker = true
                    }
                    if UIImagePickerController.isSourceTypeAvailable(.camera) {
                        UploadActionButton(title: String(localized: "Camera"), icon: "camera", tint: .green) {
                            showCamera = true
                        }
                    }
                    UploadActionButton(title: String(localized: "Paste"), icon: "doc.on.clipboard", tint: .purple) {
                        handlePasteUpload()
                    }
                }
            }
        }
        .frame(maxWidth: .infinity)
        .padding()
        #endif
    }

    private func handleFilePicker(_ result: Result<[URL], Error>) {
        guard case .success(let urls) = result, !urls.isEmpty else { return }
        Task {
            for url in urls {
                guard url.startAccessingSecurityScopedResource() else { continue }
                defer { url.stopAccessingSecurityScopedResource() }
                guard let data = try? Data(contentsOf: url) else { continue }
                let _ = await viewModel.uploadFile(
                    data: data,
                    filename: url.lastPathComponent,
                    context: modelContext
                )
            }
        }
    }

    private func batchDeleteSelectedFiles() async {
        let filesToDelete = selectedFiles
        var failCount = 0
        for file in filesToDelete {
            let success = await viewModel.deleteFile(file, context: modelContext)
            if !success { failCount += 1 }
        }
        selectedFileIDs.removeAll()
        if failCount > 0 {
            viewModel.errorMessage = String(localized: "\(failCount) file(s) failed to delete")
        }
    }

    private func handlePasteUpload() {
        guard let imageData = ImageConverter.clipboardImageData() else {
            viewModel.errorMessage = String(localized: "No image found in clipboard")
            return
        }
        Task {
            let (data, filename) = convertForUpload(imageData)
            let _ = await viewModel.uploadFile(data: data, filename: filename, context: modelContext)
        }
    }

    /// Convert image data to user's preferred format (WebP or PNG), with fallback.
    private func convertForUpload(_ imageData: Data) -> (Data, String) {
        let preferred = ImageConverter.preferredFormat

        switch preferred {
        case .webp:
            if let webpData = ImageConverter.toWebPLossless(data: imageData) {
                return (webpData, ImageConverter.pasteFilename(format: .webp))
            }
            // Fallback to PNG
            if let pngData = ImageConverter.toPNG(data: imageData) {
                return (pngData, ImageConverter.pasteFilename(format: .png))
            }
            return (imageData, ImageConverter.pasteFilename(format: .png))

        case .png:
            if let pngData = ImageConverter.toPNG(data: imageData) {
                return (pngData, ImageConverter.pasteFilename(format: .png))
            }
            return (imageData, ImageConverter.pasteFilename(format: .png))
        }
    }

    #if os(iOS)
    private func handlePhotoPicker(_ item: PhotosPickerItem) {
        Task {
            // Try loading as UIImage first for reliable HEIC→JPG conversion
            if let image = try? await item.loadTransferable(type: ImageTransferable.self) {
                let (data, filename) = convertUIImageForUpload(image.image)
                let _ = await viewModel.uploadFile(data: data, filename: filename, context: modelContext)
            } else if let data = try? await item.loadTransferable(type: Data.self) {
                // Fallback to raw data for non-image files
                let (converted, filename) = convertForUpload(data)
                let _ = await viewModel.uploadFile(data: converted, filename: filename, context: modelContext)
            }
            selectedPhoto = nil
        }
    }

    private func handleCameraCapture(_ image: UIImage) {
        Task {
            let (data, filename) = convertUIImageForUpload(image)
            let _ = await viewModel.uploadFile(data: data, filename: filename, context: modelContext)
        }
    }

    private func handleVideoCapture(_ url: URL) {
        Task {
            let df = DateFormatter()
            df.dateFormat = "yyyyMMdd-HHmmss"
            let timestamp = df.string(from: .now)

            // Convert MOV to MP4 for better compatibility
            if let mp4URL = await convertToMP4(source: url) {
                guard let data = try? Data(contentsOf: mp4URL) else {
                    viewModel.errorMessage = String(localized: "Failed to read video file")
                    return
                }
                let filename = "video-\(timestamp).mp4"
                let _ = await viewModel.uploadFile(data: data, filename: filename, context: modelContext)
                try? FileManager.default.removeItem(at: mp4URL)
            } else {
                // Fallback: upload original file if conversion fails
                guard let data = try? Data(contentsOf: url) else {
                    viewModel.errorMessage = String(localized: "Failed to read video file")
                    return
                }
                let ext = url.pathExtension.isEmpty ? "mov" : url.pathExtension
                let filename = "video-\(timestamp).\(ext)"
                let _ = await viewModel.uploadFile(data: data, filename: filename, context: modelContext)
            }
            try? FileManager.default.removeItem(at: url)
        }
    }

    /// Convert video to MP4 (H.264) using AVAssetExportSession.
    private func convertToMP4(source: URL) async -> URL? {
        let asset = AVURLAsset(url: source)
        guard let session = AVAssetExportSession(asset: asset, presetName: AVAssetExportPresetHighestQuality) else {
            return nil
        }

        let outputURL = FileManager.default.temporaryDirectory
            .appendingPathComponent(UUID().uuidString)
            .appendingPathExtension("mp4")

        do {
            try await session.export(to: outputURL, as: .mp4)
            return outputURL
        } catch {
            try? FileManager.default.removeItem(at: outputURL)
            return nil
        }
    }

    /// Convert a UIImage to JPG for maximum compatibility.
    private func convertUIImageForUpload(_ image: UIImage) -> (Data, String) {
        let df = DateFormatter()
        df.dateFormat = "yyyyMMdd-HHmmss"
        let timestamp = df.string(from: .now)

        if let jpegData = image.jpegData(compressionQuality: 0.9) {
            return (jpegData, "photo-\(timestamp).jpg")
        }
        // Fallback to PNG if JPEG fails
        if let pngData = image.pngData() {
            return (pngData, "photo-\(timestamp).png")
        }
        return (Data(), "photo-\(timestamp).jpg")
    }
    #endif
}

// MARK: - macOS Drop Zone

#if os(macOS)
struct DropZoneView: View {
    let isLoading: Bool
    let progress: Double
    let onDrop: ([URL]) -> Void
    let onTap: () -> Void
    @State private var isTargeted = false

    var body: some View {
        VStack(spacing: 12) {
            if isLoading {
                ProgressView(value: progress) {
                    Text(String(localized: "Uploading... \(Int(progress * 100))%"))
                }
            } else {
                Image(systemName: "arrow.up.doc")
                    .font(.system(size: 32))
                    .foregroundStyle(.secondary)
                Text(String(localized: "Drop files here, click to browse, or ⌘V to paste"))
                    .font(.subheadline)
                    .foregroundStyle(.secondary)
            }
        }
        .frame(height: 120)
        .frame(maxWidth: 560)
        .background(
            RoundedRectangle(cornerRadius: 12)
                .stroke(style: StrokeStyle(lineWidth: 2, dash: [8]))
                .foregroundStyle(isTargeted ? Color.accentColor : .secondary.opacity(0.3))
        )
        .background(isTargeted ? Color.accentColor.opacity(0.05) : Color.clear, in: RoundedRectangle(cornerRadius: 12))
        .frame(maxWidth: .infinity, alignment: .center)
        .onTapGesture { onTap() }
        .dropDestination(for: URL.self) { urls, _ in
            onDrop(urls)
            return true
        } isTargeted: { targeted in
            isTargeted = targeted
        }
    }
}
#endif

// MARK: - Uploaded File Row

struct UploadedFileRow: View {
    let file: UploadedFile
    let linkDisplayType: LinkDisplayType
    let onDelete: () -> Void

    private var sizeInfo: String {
        var parts = [file.size.formattedFileSize]
        if let w = file.width, let h = file.height, w > 0, h > 0 {
            parts.append("\(w)×\(h)")
        }
        if let duration = file.duration, duration > 0 {
            parts.append(formatDuration(duration))
        }
        return parts.joined(separator: " · ")
    }

    private func formatDuration(_ seconds: Double) -> String {
        let totalSeconds = Int(seconds)
        let h = totalSeconds / 3600
        let m = (totalSeconds % 3600) / 60
        let s = totalSeconds % 60
        if h > 0 {
            return String(format: "%d:%02d:%02d", h, m, s)
        }
        return String(format: "%d:%02d", m, s)
    }

    private var displayedLink: String {
        linkDisplayType.formatted(file: file)
    }

    private var isClickableURL: Bool {
        linkDisplayType == .directLink || linkDisplayType == .sharePage
    }

    var body: some View {
        HStack(spacing: 12) {
            // Thumbnail
            CachedThumbnailView(identifier: file.url, size: 44)

            VStack(alignment: .leading, spacing: 4) {
                HStack {
                    Text(file.filename)
                        .font(.subheadline.weight(.medium))
                        .lineLimit(1)
                    Spacer()
                    Text(file.createdAt.relativeFormatted)
                        .font(.caption2)
                        .foregroundStyle(.tertiary)
                }

                HStack(spacing: 4) {
                    if isClickableURL, let url = URL(string: displayedLink) {
                        Link(displayedLink, destination: url)
                            .font(.caption)
                            .lineLimit(1)
                    } else {
                        Text(displayedLink)
                            .font(.caption.monospaced())
                            .foregroundStyle(.secondary)
                            .lineLimit(1)
                            .textSelection(.enabled)
                    }

                    CopyButton(text: displayedLink)
                }

                Text(sizeInfo)
                    .font(.caption2)
                    .foregroundStyle(.tertiary)
            }
        }
        .padding(.vertical, 4)
        .contextMenu {
            Button(String(localized: "Direct Link")) {
                ClipboardService.copy(file.url)
            }

            Button(String(localized: "Share Page")) {
                ClipboardService.copy(file.page)
            }

            Divider()

            Menu("BBCode") {
                Button("BBCode") {
                    ClipboardService.copy(
                        LinkFormatter.bbcode(filename: file.filename, directURL: file.url)
                    )
                }
                Button("BBCode w/ Link") {
                    ClipboardService.copy(
                        LinkFormatter.bbcodeWithLink(filename: file.filename, pageURL: file.page, directURL: file.url)
                    )
                }
                Button("BBCode w/ Direct Link") {
                    ClipboardService.copy(
                        LinkFormatter.bbcodeDirectLink(filename: file.filename, directURL: file.url)
                    )
                }
            }

            Menu("HTML") {
                Button("HTML") {
                    ClipboardService.copy(
                        LinkFormatter.html(filename: file.filename, directURL: file.url)
                    )
                }
                Button("HTML w/ Link") {
                    ClipboardService.copy(
                        LinkFormatter.htmlWithLink(filename: file.filename, pageURL: file.page, directURL: file.url)
                    )
                }
                Button("HTML w/ Direct Link") {
                    ClipboardService.copy(
                        LinkFormatter.htmlDirectLink(filename: file.filename, directURL: file.url)
                    )
                }
            }

            Menu("Markdown") {
                Button("Markdown") {
                    ClipboardService.copy(
                        LinkFormatter.markdown(filename: file.filename, directURL: file.url)
                    )
                }
            }

            Divider()

            Button(String(localized: "Open in Browser")) {
                if let url = URL(string: file.page) {
                    #if os(macOS)
                    NSWorkspace.shared.open(url)
                    #else
                    UIApplication.shared.open(url)
                    #endif
                }
            }

            Divider()

            Button(String(localized: "Delete"), role: .destructive) { onDelete() }
        }
    }
}

// MARK: - Batch Links View

struct BatchLinksView: View {
    @Environment(\.dismiss) private var dismiss
    let files: [UploadedFile]
    let linkDisplayType: LinkDisplayType
    let onDismiss: () -> Void
    @State private var batchType: LinkDisplayType
    @State private var copied = false

    init(files: [UploadedFile], linkDisplayType: LinkDisplayType, onDismiss: @escaping () -> Void) {
        self.files = files
        self.linkDisplayType = linkDisplayType
        self.onDismiss = onDismiss
        self._batchType = State(initialValue: linkDisplayType)
    }

    private var batchText: String {
        files.map { batchType.formatted(file: $0) }.joined(separator: "\n")
    }

    var body: some View {
        NavigationStack {
            VStack(spacing: 0) {
                // Format picker
                HStack {
                    Text(String(localized: "Format"))
                        .font(.subheadline.weight(.medium))
                    Picker("", selection: $batchType) {
                        ForEach(LinkDisplayType.allCases) { type in
                            Text(type.rawValue).tag(type)
                        }
                    }
                    .labelsHidden()
                }
                .padding(.horizontal)
                .padding(.vertical, 8)

                // Links preview
                ScrollView {
                    Text(batchText)
                        .font(.caption.monospaced())
                        .textSelection(.enabled)
                        .frame(maxWidth: .infinity, alignment: .leading)
                        .padding()
                }
                .background(Color.secondary.opacity(0.05))

                Divider()

                // Actions
                HStack {
                    Button(String(localized: "Clear Selection"), role: .destructive) {
                        onDismiss()
                    }

                    Spacer()

                    Button(action: {
                        ClipboardService.copy(batchText)
                        copied = true
                        DispatchQueue.main.asyncAfter(deadline: .now() + 1.5) {
                            copied = false
                        }
                    }) {
                        HStack(spacing: 4) {
                            Image(systemName: copied ? "checkmark" : "doc.on.doc")
                                .contentTransition(.symbolEffect(.replace))
                            Text(copied ? String(localized: "Copied!") : String(localized: "Copy All"))
                        }
                    }
                    .buttonStyle(.borderedProminent)
                }
                .padding()
            }
            .navigationTitle(String(localized: "Batch Copy Links"))
            #if os(iOS)
            .navigationBarTitleDisplayMode(.inline)
            #endif
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button(String(localized: "Done")) { dismiss() }
                }
            }
            #if os(macOS)
            .frame(minWidth: 500, minHeight: 400)
            #endif
        }
    }
}

// MARK: - iOS Camera & Image Helpers

#if os(iOS)
/// Compact icon + label button for the iOS upload area.
struct UploadActionButton: View {
    let title: String
    let icon: String
    let tint: Color
    let action: () -> Void

    var body: some View {
        Button(action: action) {
            VStack(spacing: 8) {
                Image(systemName: icon)
                    .font(.title3)
                    .frame(width: 52, height: 52)
                    .background(tint.opacity(0.12), in: Circle())
                    .foregroundStyle(tint)
                Text(title)
                    .font(.caption)
                    .foregroundStyle(.primary)
            }
        }
        .buttonStyle(.plain)
    }
}

/// Transferable wrapper to load UIImage from PhotosPicker.
struct ImageTransferable: Transferable {
    let image: UIImage

    static var transferRepresentation: some TransferRepresentation {
        DataRepresentation(importedContentType: .image) { data in
            guard let image = UIImage(data: data) else {
                throw TransferError.importFailed
            }
            return ImageTransferable(image: image)
        }
    }

    enum TransferError: Error {
        case importFailed
    }
}

/// UIImagePickerController wrapper for camera capture (photo + video).
struct CameraView: UIViewControllerRepresentable {
    let onCapture: (UIImage) -> Void
    let onVideoCapture: (URL) -> Void
    @Environment(\.dismiss) private var dismiss

    func makeUIViewController(context: Context) -> UIImagePickerController {
        let picker = UIImagePickerController()
        picker.sourceType = .camera
        picker.mediaTypes = [UTType.image.identifier, UTType.movie.identifier]
        picker.videoQuality = .typeHigh
        picker.delegate = context.coordinator
        return picker
    }

    func updateUIViewController(_ uiViewController: UIImagePickerController, context: Context) {}

    func makeCoordinator() -> Coordinator {
        Coordinator(onCapture: onCapture, onVideoCapture: onVideoCapture, dismiss: dismiss)
    }

    class Coordinator: NSObject, UIImagePickerControllerDelegate, UINavigationControllerDelegate {
        let onCapture: (UIImage) -> Void
        let onVideoCapture: (URL) -> Void
        let dismiss: DismissAction

        init(onCapture: @escaping (UIImage) -> Void, onVideoCapture: @escaping (URL) -> Void, dismiss: DismissAction) {
            self.onCapture = onCapture
            self.onVideoCapture = onVideoCapture
            self.dismiss = dismiss
        }

        func imagePickerController(_ picker: UIImagePickerController, didFinishPickingMediaWithInfo info: [UIImagePickerController.InfoKey: Any]) {
            if let mediaType = info[.mediaType] as? String {
                if mediaType == UTType.movie.identifier, let url = info[.mediaURL] as? URL {
                    onVideoCapture(url)
                } else if let image = info[.originalImage] as? UIImage {
                    onCapture(image)
                }
            }
            dismiss()
        }

        func imagePickerControllerDidCancel(_ picker: UIImagePickerController) {
            dismiss()
        }
    }
}
#endif
