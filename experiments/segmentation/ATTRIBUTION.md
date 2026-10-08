# Evaluation assets

| Asset | Author / source | Terms |
|---|---|---|
| Astronaut photograph in contact sheet | NASA; [scikit-image astronaut](https://scikit-image.org/docs/stable/api/skimage.data.html#skimage.data.astronaut) | Public domain / no known copyright restrictions |
| Chelsea cat photograph in contact sheet | Stefan van der Walt; [scikit-image chelsea](https://scikit-image.org/docs/stable/api/skimage.data.html#skimage.data.chelsea) | CC0 |
| Coffee photograph in contact sheet | Rachel Michetti; [scikit-image coffee](https://scikit-image.org/docs/stable/api/skimage.data.html#skimage.data.coffee) | CC0 |
| Synthetic fixtures, overlays and evaluation code | 1puni and contributors, created with AI coding assistance | MIT OR Apache-2.0, repository root licenses |

CC0 notice: the photographer waived copyright and related rights to the extent
permitted by law. [CC0 1.0 legal text](https://creativecommons.org/publicdomain/zero/1.0/legalcode).
The installed scikit-image 0.26.0 `data/_fetchers.py` records these source notices.
The photographs and derived masks are evaluation material, not model training data.

Downloads remain outside Git. [MobileSAM](https://github.com/ChaoningZhang/MobileSAM)
and [EdgeTAM](https://github.com/facebookresearch/EdgeTAM) publish Apache-2.0 code;
EdgeTAM explicitly includes its checkpoints. The tested ONNX conversions are
[Acly/MobileSAM](https://huggingface.co/Acly/MobileSAM) and
[onnx-community/EdgeTAM-ONNX](https://huggingface.co/onnx-community/EdgeTAM-ONNX).
Pinned conversion URLs/hashes are retained in evidence/models.json. A production
distribution must retain upstream and conversion notices with its model assets;
this experiment does not bundle or relicense those weights.
