using CSharpLargeApi.Domain.Interfaces;

namespace CSharpLargeApi.Domain.Entities;

/// <summary>
/// Represents an uploaded media file (image, document, video).
/// Stores metadata about the file including size, type, and storage location.
/// </summary>
public class Media : IEntity
{
    /// <summary>
    /// Gets the unique identifier for this media item.
    /// </summary>
    public Guid Id { get; private set; }

    /// <summary>
    /// Gets the timestamp when this media was uploaded.
    /// </summary>
    public DateTime CreatedAt { get; private set; }

    /// <summary>
    /// Gets the timestamp when this media was last modified.
    /// </summary>
    public DateTime? UpdatedAt { get; private set; }

    /// <summary>
    /// Gets the original filename as uploaded by the user.
    /// </summary>
    public string FileName { get; private set; } = string.Empty;

    /// <summary>
    /// Gets the MIME content type of this media file.
    /// </summary>
    public string ContentType { get; private set; } = string.Empty;

    /// <summary>
    /// Gets the file size in bytes.
    /// </summary>
    public long FileSizeBytes { get; private set; }

    /// <summary>
    /// Gets the storage path or URL where this media is stored.
    /// </summary>
    public string StoragePath { get; private set; } = string.Empty;

    /// <summary>
    /// Gets the ID of the user who uploaded this media.
    /// </summary>
    public Guid UploadedById { get; private set; }

    /// <summary>
    /// Gets the optional alt text for accessibility.
    /// </summary>
    public string? AltText { get; private set; }

    /// <summary>
    /// Creates a new media record for an uploaded file.
    /// </summary>
    public static Media Create(Guid uploadedById, string fileName, string contentType, long fileSizeBytes, string storagePath)
    {
        return new Media
        {
            Id = Guid.NewGuid(),
            UploadedById = uploadedById,
            FileName = fileName,
            ContentType = contentType,
            FileSizeBytes = fileSizeBytes,
            StoragePath = storagePath,
            CreatedAt = DateTime.UtcNow
        };
    }

    /// <summary>
    /// Updates the alt text for this media item.
    /// </summary>
    public void UpdateAltText(string? altText)
    {
        AltText = altText;
        UpdatedAt = DateTime.UtcNow;
    }
}
